#[link(name = "squiggle",
       vers = "0.1",
       uuid = "7f09eec3-932c-4096-9c3f-7f0b22c18f2a")];

extern mod std;
extern mod extra;

use eval::Eval;
use gen::*;
use webapi::*;

use std::hashmap::HashMap;
use std::io;
use std::io::WriterUtil;
use std::os;
use std::path::Path;
use std::rand::{Rng, RngUtil};
use std::vec;
use extra::sort;
use extra::time;

pub mod eval;
pub mod gen;
pub mod parse;
pub mod program;
pub mod webapi;
pub mod compile;

fn main() {
    let args = os::args();
    if args.len() < 2 {
        println("usage: squiggle COMMAND");
        return;
    }
    match args[1] {
        ~"status" => status(),
        ~"train" => {
            if args.len() < 3 {
                println("error: missing training size");
            } else {
                let folding = if args.len() == 4 {
                    match args[3] {
                        ~"fold" => Fold,
                        ~"tfold" => Tfold,
                        _ => {
                            println("error: bad folding value, using no folds");
                            Empty
                        }
                    }
                } else {
                    Empty
                };
                train(FromStr::from_str(args[2]).expect("bad size"),
                      folding,
                      false);
            }
        }
        ~"faketrain" => {
            let progs = do args.slice_from(2).iter().transform |string| {
                use parse::Parse;
                string.parse()
            }.collect();

            faketrain(progs)
        }
        ~"localtrain" => {
            if args.len() < 3 {
                println("error: missing training size");
            } else {
                let folding = if args.len() == 4 {
                    match args[3] {
                        ~"fold" => Fold,
                        ~"tfold" => Tfold,
                        _ => {
                            println("error: bad folding value, using no folds");
                            Empty
                        }
                    }
                } else {
                    Empty
                };
                train(FromStr::from_str(args[2]).expect("bad size"),
                      folding,
                      true);
            }
        }
        ~"problems" => {
            let count = if args.len() >= 3 {
                FromStr::from_str(args[2]).expect("bad count")
            } else {
                1_000_000_000 // run forever
            };
            let filter = if args.len() >= 4 {
                match args[3] {
                    ~"fold" => Folded,
                    ~"tfold" => Tfolded,
                    ~"all" => All,
                    ~"unfold" => Unfolded,
                    ~"bonus" => Bonus,
                    ~"nobonus" => NoBonus,
                    _ => {
                        println("error: bad filter value, using no filter");
                        All
                    }
                }
            } else {
                All
            };
            let min_size = if args.len() >= 5 {
                FromStr::from_str(args[4]).expect("bad min size")
            } else {
                0
            };
            problems(count, filter, min_size)
        }
        ~"showprobs" => show_problems(),
        ~"eval" => {
            let prog = {
                use parse::Parse;
                args[2].parse()
            };
            eval(prog);
        }
        _ => println("error: unknown command"),
    }
}

fn status() {
    let status = WebApi::new().get_status_blocking();
    printfln!("%?", status);
}

fn train(size: u8, operator: TrainOperator, local: bool) {
    let mut api = WebApi::new();
    let mut local_api = FakeApi::new(~[]);
    let mut stats = Statistics::new();
    let mut gen = RandomGen::blank();

    loop {
        let prob = api.get_training_blocking(size, operator);
        let mut s = ~"";
        s.push_str(fmt!("TRAIN: -- %u -- %s -- %s\n",
                        prob.problem.size as uint,
                        prob.problem.operators.to_str(),
                        prob.problem.id));
        s.push_str(fmt!("GOLD: %s\n\n", prob.challenge));
        println(s);
        let writer = io::file_writer(&Path("train.log"), [io::Append]).unwrap();
        writer.write_str(s);

        if local {
            println("solving locally");
            local_api.add_prog(prob.problem.id, {
                use parse::Parse;
                prob.challenge.parse()
            });
            solve_problem(prob.problem, &mut local_api, &mut stats, &mut gen);
        } else {
            println("solving remotely");
            solve_problem(prob.problem, &mut api, &mut stats, &mut gen);
        }
    }
}

fn faketrain(progs: ~[program::Program]) {
    let mut api = FakeApi::new(progs);
    let mut stats = Statistics::new();
    let mut gen = RandomGen::blank();

    while api.has_programs() {
        // the args are ignored anyway
        let prob = api.get_training_blocking(0, Empty);

        printfln!("FAKETRAIN: -- %u -- %s -- %s",
                  prob.problem.size as uint,
                  prob.problem.operators.to_str(),
                  prob.problem.id);

        solve_problem(prob.problem, &mut api, &mut stats, &mut gen);
    }
}

fn problems(count: uint, filter: ProblemFilter, min_size: u8) {
    let mut api = WebApi::new();
    let mut stats = Statistics::new();
    let mut gen = RandomGen::blank();

    let probs = api.get_problems_blocking();
    // TODO filter problems by train operator.
    let mut unsolved_probs: ~[RealProblem] = probs.consume_iter()
        .filter(|p| !p.solved && p.time_left.map_default(true, |&n| n > 0.0))
        .filter(|p| match filter {
            All => true,
            Tfolded => p.problem.operators.tfold,
            Unfolded => (!p.problem.operators.fold &&
                         !p.problem.operators.tfold &&
                         !p.problem.operators.bonus),
            Folded => p.problem.operators.fold,
            Bonus => p.problem.operators.bonus,
            NoBonus => !p.problem.operators.bonus,
        })
        .filter(|p| p.problem.size >= min_size)
        .collect();
    sort::tim_sort(unsolved_probs);

    for prob in unsolved_probs.consume_iter().take_(count) {
        printfln!("PROBLEM: -- %u -- %s -- %s",
                  prob.problem.size as uint,
                  prob.problem.operators.to_str(),
                  prob.problem.id);

        solve_problem(prob.problem, &mut api, &mut stats, &mut gen);
    }
}

fn solve_problem<A: Api>(problem: Problem, api: &mut A, stats: &mut Statistics,
                         gen: &mut RandomGen) {
    let pairs = fetch_n_random_testcases(problem.clone(), 50, api);

    stats.start();
    gen.reset(problem.clone(), pairs);

    loop {
        match gen.next() {
            Some(candidate) => {
                println(candidate.to_str());
                info!(candidate);
                match api.guess_blocking(problem.clone(), candidate.to_str()) {
                    Win => {
                        println("win!");
                        break
                    }
                    Mismatch(input, real, ours) => {
                        printfln!("P(%?) == %? != %?", input, real, ours);

                        let mut pairs = fetch_n_random_testcases(problem.clone(), 50, api);
                        pairs.push((input, real));

                        gen.more_constraints(pairs);
                    }
                    Error(s) => {
                        printfln!("Error occured: %s", s);
                    }
                }
            }
            None => {
                println("Timed out :(");
                break
            }
        }
    }

    stats.end();
    stats.report();
}

fn show_problems() {
    let mut api = WebApi::new();
    let mut probs: ~[RealProblem] = api.get_problems_blocking().consume_iter().collect();
    sort::tim_sort(probs);

    let mut stats = HashMap::new::<uint,uint>();
    let mut failed = 0u;
    let mut solved = 0u;
    let mut total = 0u;

    for prob in probs.iter() {
        let status = match (prob.solved, prob.time_left) {
            (true, _) => {
                solved += 1;
                "SOLVED"
            }
            (false, None) => "UNSOLVED",
            (false, Some(0f)) => {
                failed += 1;
                "FAILED"
            }
            (false, Some(_)) => "IN PROGRESS",
        };
        total += 1;
        do stats.insert_or_update_with(prob.problem.size as uint, 1) |_, v| { *v += 1; };

        printfln!("%s -- %u -- %s -- %s",
                  status,
                  prob.problem.size as uint,
                  prob.problem.operators.to_str(),
                  prob.problem.id);
    }

    println("SIZES:");
    for (k, v) in stats.iter() {
        printfln!("\tsize %u: %u", *k, *v);
    }


    printfln!("STATS: %u (%u%%) solved -- %u (%u%%) failed",
              solved, ((solved as float) / (total as float) * 100f) as uint,
              failed, ((failed as float) / (total as float) * 100f) as uint);
}

fn eval(program: program::Program) {
    let mut api = WebApi::new();

    printfln!("EVAL: -- %s", program.to_str());


    let mut rng = std::rand::task_rng();
    let inputs: ~[u64] = std::vec::from_fn(50, |_| rng.gen());
    let local_outputs: ~[u64] = inputs.iter().transform(|&x| program.eval(x)).collect();
    let remote_outputs = api.eval_program_blocking(program, inputs.clone()).unwrap();

    let mut all_match = true;
    for i in range(0, local_outputs.len()) {
        if local_outputs[i] != remote_outputs[i] {
            all_match = false;
            printfln!("mismatch: P(%u) = %u != %u",
                      inputs[i] as uint,
                      local_outputs[i] as uint,
                      remote_outputs[i] as uint);
        }
    }
    if all_match {
        println("OK");
    } else {
        println("NOT OK");
    }
}

enum ProblemFilter {
    All,
    Tfolded,
    Unfolded,
    Folded,
    Bonus,
    NoBonus,
}

struct Statistics {
    start: u64,
    cursor: uint,
    history: ~[u64],
    max_size: uint,
    size: uint,
}

impl Statistics {
    pub fn new() -> Statistics {
        Statistics {
            start: 0,
            cursor: 0,
            history: vec::from_elem(25, 0u64),
            max_size: 25,
            size: 0,
        }
    }

    pub fn start(&mut self) {
        assert!(self.start == 0);
        self.start = time::precise_time_ns();
    }

    pub fn end(&mut self) {
        let ns = time::precise_time_ns() - self.start;
        self.start = 0;
        self.history[self.cursor] = ns;
        self.cursor = (self.cursor + 1) % 25;
        if self.size < 25 {
            self.size += 1;
        }
    }

    pub fn report(&self) {
        let mut sum = 0;
        for v in self.history.iter() {
            sum += *v;
        }
        let avg = (sum as f64) / (self.size as f64);
        printfln!("stats: avg candidate time is %ums", (avg / 1000000f64) as uint);
    }
}

fn fetch_n_random_testcases<A: Api>(p: Problem, n: uint, api: &mut A) -> ~[(u64, u64)] {
    let mut rng = std::rand::task_rng();
    let tests = std::vec::from_fn(n, |_| rng.gen());

    let constraints = api.eval_blocking(p, tests.clone()).expect("coulnd't eval tests");

    tests.consume_iter().zip(constraints.consume_iter()).collect()
}

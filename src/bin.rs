#[link(name = "squiggle",
       vers = "0.1",
       uuid = "7f09eec3-932c-4096-9c3f-7f0b22c18f2a")];

extern mod std;
extern mod extra;

use eval::Eval;
use gen::*;
use webapi::*;

use std::os;
use std::rand::{Rng, RngUtil, XorShiftRng};
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
                      folding);
            }
        }
        ~"problems" => problems(),
        ~"showprobs" => show_problems(),
        _ => println("error: unknown command"),
    }
}

fn status() {
    let status = WebApi::new().get_status_blocking();
    printfln!("%?", status);
}

fn train(size: u8, operator: TrainOperator) {
    let mut api = WebApi::new();
    let mut stats = Statistics::new();
    let mut rng = seeded_rng();

    loop {
        let prob = api.get_training_blocking(size, operator);
        printfln!("TRAIN: -- %u -- %s -- %s",
                  prob.problem.size as uint,
                  prob.problem.operators.to_str(),
                  prob.problem.id);

        solve_problem(prob.problem, &mut api, &mut stats, &mut rng);
    }
}

fn problems() {
    let mut api = WebApi::new();
    let mut stats = Statistics::new();
    let mut rng = seeded_rng();

    let probs = api.get_problems_blocking();
    let mut unsolved_probs: ~[RealProblem] = probs.consume_iter()
        .filter(|p| !p.solved && p.time_left.map_default(true, |&n| n > 0.0))
        .collect();
    sort::tim_sort(unsolved_probs);

    for prob in unsolved_probs.consume_iter() {
        printfln!("attempting problem %s (%u)", prob.problem.id, prob.problem.size as uint);

        solve_problem(prob.problem, &mut api, &mut stats, &mut rng);
    }
}

fn solve_problem<R: Rng>(problem: Problem, api: &mut WebApi, stats: &mut Statistics, rng: &mut R) {
    let pairs = fetch_n_random_testcases(problem.clone(), 50, api, rng);

    stats.start();
    let mut gen = RandomGen::new(problem.clone(), pairs);

    'next_candidate: loop {
        let candidate = gen.next();

        println(candidate.to_str());
        info!(candidate);
        match api.guess_blocking(problem.clone(), candidate.to_str()) {
            Win => {
                println("win!");
                break 'next_candidate;
            }
            Mismatch(input, real, ours) => {
                printfln!("P(%?) == %? != %?", input, real, ours);

                let mut pairs = fetch_n_random_testcases(problem.clone(), 50, api, rng);
                pairs.push((input, real));

                gen.more_constraints(pairs);
                loop 'next_candidate;
            }
            Error(s) => {
                printfln!("Error occured: %s", s);
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

    let mut stats = vec::from_elem(28, 0u);
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
        stats[prob.problem.size - 3] += 1;

        printfln!("%s -- %u -- %s -- %s",
                  status,
                  prob.problem.size as uint,
                  prob.problem.operators.to_str(),
                  prob.problem.id);
    }

    println("SIZES:");
    for i in range(3, 31) {
        printfln!("\tsize %i: %u", i, stats[i - 3]);
    }


    printfln!("STATS: %u (%u%%) solved -- %u (%u%%) failed",
              solved, ((solved as float) / (total as float) * 100f) as uint,
              failed, ((failed as float) / (total as float) * 100f) as uint);
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

fn seeded_rng() -> XorShiftRng {
    let mut seed_rng = std::rand::task_rng();
    XorShiftRng::new_seeded(
        seed_rng.gen::<u32>(),
        seed_rng.gen::<u32>(),
        seed_rng.gen::<u32>(),
        seed_rng.gen::<u32>())
}

fn fetch_n_random_testcases<R: Rng>(p: Problem, n: uint, api: &mut WebApi, rng: &mut R)
    -> ~[(u64, u64)] {
    let tests = std::vec::from_fn(n, |_| rng.gen());

    let constraints = api.eval_blocking(p, tests.clone()).expect("coulnd't eval tests");

    tests.consume_iter().zip(constraints.consume_iter()).collect()
}

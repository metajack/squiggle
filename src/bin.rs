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
            if args.len() != 3 {
                println("error: missing training size");
            } else {
                train(FromStr::from_str(args[2]).expect("bad size"), Empty);
            }
        }
        ~"problems" => problems(),
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
        printfln!("TRAIN: %s (%u)", prob.problem.id, prob.problem.size as uint);

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
    let mut gen = NaiveGen::new(problem.size, problem.operators, pairs);

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

#[link(name = "squiggle",
       vers = "0.1",
       uuid = "7f09eec3-932c-4096-9c3f-7f0b22c18f2a")];

extern mod std;
extern mod extra;

use eval::Eval;
use gen::*;
use webapi::*;

use std::os;
use std::rand::{RngUtil, XorShiftRng};
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
    println(status.to_str());
}

fn train(size: u8, operator: TrainOperator) {
    let mut api = WebApi::new();
    let mut stats = Statistics::new();
    let mut rng = seeded_rng();

    loop {
        let prob = api.get_training_blocking(size, operator);
        printfln!("TRAIN: %s (%u)", prob.problem.id, prob.problem.size as uint);

        let mut tests = ~[];
        for _ in range(0, 50) {
            tests.push(rng.gen::<u64>());
        }

        let constraints = api.eval_blocking(prob.problem.clone(), tests.clone()).expect("coulnd't eval tests");
        let pairs = tests.consume_iter().zip(constraints.consume_iter()).collect();

        stats.start();
        let mut gen = NaiveGen::new(prob.problem.size, prob.problem.operators, pairs);

        let candidate = gen.next();
        stats.end();

        println(candidate.to_str());
        printfln!(api.guess_blocking(prob.problem.clone(), candidate.to_str()));
        stats.report();
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

    for prob in unsolved_probs.iter() {
        printfln!("attempting problem %s (%u)", prob.problem.id, prob.problem.size as uint);

        let mut tests = ~[];
        for _ in range(0, 50) {
            tests.push(rng.gen::<u64>());
        }

        let constraints = api.eval_blocking(prob.problem.clone(), tests.clone()).expect("coulnd't eval tests");
        let pairs = tests.consume_iter().zip(constraints.consume_iter()).collect();

        stats.start();
        let mut gen = NaiveGen::new(prob.problem.size, prob.problem.operators, pairs);

        let candidate = gen.next();
        stats.end();

        println(candidate.to_str());
        printfln!(api.guess_blocking(prob.problem.clone(), candidate.to_str()));
        stats.report();
    }
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
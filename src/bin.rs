#[link(name = "squiggle",
       vers = "0.1",
       uuid = "7f09eec3-932c-4096-9c3f-7f0b22c18f2a")];

extern mod std;
extern mod extra;

use eval::Eval;
use gen::*;
use webapi::*;

use std::os;
use std::rand::RngUtil;
use extra::sort;

pub mod eval;
pub mod gen;
pub mod parse;
pub mod program;
pub mod webapi;
pub mod compile;

fn main() {
    let args = os::args();
    if args.len() != 2 {
        println("usage: squiggle COMMAND");
        return;
    }
    match args[1] {
        ~"status" => status(),
        ~"problems" => problems(),
        _ => println("error: unknown command"),
    }
}

fn status() {
    let status = WebApi::new().get_status_blocking();
    println(status.to_str());
}

fn problems() {
    let mut api = WebApi::new();
    let probs = api.get_problems_blocking();
    let mut unsolved_probs: ~[RealProblem] = probs.consume_iter()
        .filter(|p| !p.solved && p.time_left.map_default(true, |&n| n > 0.0))
        .collect();
    sort::tim_sort(unsolved_probs);

    for prob in unsolved_probs.iter() {
        printfln!("attempting problem %s (%u)", prob.id, prob.size as uint);

        let mut rng = std::rand::task_rng();
        let mut tests = ~[];
        for _ in range(0, 50) {
            tests.push(rng.gen::<u64>());
        }

        let constraints = api.eval_blocking(prob.clone(), tests.clone()).expect("coulnd't eval tests");
        let pairs = tests.consume_iter().zip(constraints.consume_iter()).collect();

        let mut gen = NaiveGen::new(prob.size, prob.operators, pairs);

        let candidate = gen.next();
        println(candidate.to_str());
        printfln!(api.guess_blocking(prob.clone(), candidate.to_str()))
    }
}

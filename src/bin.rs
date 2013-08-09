#[link(name = "squiggle",
       vers = "0.1",
       uuid = "7f09eec3-932c-4096-9c3f-7f0b22c18f2a")];

extern mod std;
extern mod extra;

use eval::Eval;
use gen::*;
use program::*;
use webapi::{Request, TrainOperators, Empty};

pub mod eval;
pub mod gen;
pub mod program;
pub mod webapi;

fn main() {
    // let status = webapi::Request::get_status();
    // println(status.to_str());

    // let prob = webapi::Request::get_training_problem(3, Empty);

    println(Program::new(~"x", ~Ident(~"x")).to_str());

    // (|x| x << 1 + x)(10)
    printfln!(Program::new(~"x", ~Op2(Plus,
                                      ~Op1(Shl1, ~Ident(~"x")),
                                      ~Ident(~"x"))).eval(10));

    let prog = Program::new(~"x", ~Fold {
            foldee: ~Ident(~"x"),
            init: ~Zero,
            accum_id: ~"y",
            next_id: ~"z",
            body: ~Op2(Or, ~Ident(~"y"), ~Ident(~"z"))
        });

    printfln!(prog.eval(0x1122334455667788))

    // some random programs
    let mut gen = NaiveGen::new();
    for _ in range(0, 5) {
        printfln!(gen.gen_prog().to_str());
        gen.reset();
    }
}

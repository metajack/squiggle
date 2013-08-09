#[link(name = "squiggle",
       vers = "0.1",
       uuid = "7f09eec3-932c-4096-9c3f-7f0b22c18f2a")];

extern mod std;
extern mod extra;

use webapi::{Request, TrainOperators, Empty};
use program::*;
use eval::Eval;

pub mod webapi;
pub mod program;
pub mod eval;

fn main() {
    // let status = webapi::Request::get_status();
    // println(status.to_str());

    // let prob = webapi::Request::get_training_problem(3, Empty);

    println(Program::new(~"x", ~Ident(~"x")).to_str());

    // (|x| x << 1 + x)(10)
    printfln!(Program::new(~"x", ~Op2(Plus,
                                      ~Op1(Shl1, ~Ident(~"x")),
                                      ~Ident(~"x"))).eval(10));
}

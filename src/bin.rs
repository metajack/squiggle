#[link(name = "squiggle",
       vers = "0.1",
       uuid = "7f09eec3-932c-4096-9c3f-7f0b22c18f2a")];

extern mod std;
extern mod extra;

use webapi::{Request, TrainOperators, Empty};

pub mod webapi;
pub mod program;

fn main() {
    let status = webapi::Request::get_status();
    println(status.to_str());

    let prob = webapi::Request::get_training_problem(3, Empty);
}

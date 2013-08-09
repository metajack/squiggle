use program::*;

use std::run::{Process, ProcessOptions};
use std::to_str::ToStr;
use std::num::{FromStrRadix,ToStrRadix};
use extra::json;
use extra::json::{Json, ToJson, Object, Number, String, List, Boolean};
use extra::treemap::TreeMap;

static SERVER: &'static str = "http://icfpc2013.cloudapp.net/";

static PRIVATE_KEY: &'static str = include_str!("private.key");

enum Request {
    Status,
    Train { size: u8, operators: TrainOperators },
}

pub enum TrainOperators {
    Empty,
    Tfold,
    Fold,
}

impl Request {
    pub fn to_url(&self) -> ~str {
        match *self {
            Status => {
                make_url("status")
            }
            Train { _ } => {
                make_url("train")
            }
        }
    }

    pub fn to_json_str(&self) -> ~str {
        match *self {
            Train { size: size, operators: ref ops } => {
                let mut obj: TreeMap<~str, Json> = TreeMap::new();
                obj.insert(~"size", size.to_json());
                obj.insert(~"operators", match *ops {
                    Empty => (~"").to_json(),
                    Tfold => (~"tfold").to_json(),
                    Fold => (~"fold").to_json(),
                });
                obj.to_json().to_str()
            }
            _ => fail!(~"not implemented"),
        }
    }

    pub fn get_status() -> StatusResponse {
        let response = get_request(Status.to_url());
        StatusResponse(response)
    }

    pub fn get_training_problem(size: u8, operators: TrainOperators) -> TrainingProblem {
        let req = Train { size: size, operators: operators };
        let response = post_request(req.to_url(), req.to_json_str());

        match *response {
            Object(obj) => {
                let challenge = get_json_str(obj, ~"challenge");
                let id = get_json_str(obj, ~"id");
                let size = get_json_num(obj, ~"size");

                let array = get_json_array(obj, ~"operators");
                let mut ops = OperatorSet::new();
                let str_ops: ~[~str] = do array.iter().transform |op| {
                    match *op {
                        String(ref s) => s.clone(),
                        _ => fail!("bad value in 'operators'"),
                    }
                }.collect();
                ops.add(str_ops);

                TrainingProblem {
                    challenge: challenge,
                    id: id,
                    size: size as u8,
                    operators: ops,
                }
            }
            _ => fail!("bad response"),
        }
    }

    pub fn get_real_problems() -> ~[RealProblem] {
        let response = match get_request(make_url("myproblems")) {
            ~List(a) => a,
            _ => fail!("bad myproblems response")
        };

        do response.consume_iter().transform |x| {
            match x {
                Object(resp) => {
                    let id = get_json_str(resp, ~"id");
                    let size = get_json_num(resp, ~"size");

                    let solved = match resp.find(&~"solved")  {
                        Some(&Boolean(x)) => x,
                        None => false,
                        _ => fail!("invalid solved boolean"),
                    };

                    let time_left = do resp.find(&~"timeLeft").map |tl| {
                        match **tl {
                            Number(x) => x,
                            _ => fail!("invalid timeLeft number")
                        }
                    };
                    let array = get_json_array(resp, ~"operators");
                    let mut ops = OperatorSet::new();
                    let str_ops = do array.iter().transform |op| {
                        match *op {
                            String(ref s) => s.clone(),
                            _ => fail!("bad value in 'operators'"),
                        }
                    }.collect();
                    ops.add(str_ops);

                    RealProblem {
                        id: id,
                        size: size as u8,
                        solved: solved,
                        time_left: time_left,
                        operators: ops
                    }
                }
                _ => fail!("invalid response")
            }
        }.collect()
    }
}

struct StatusResponse(~Json);

impl ToStr for StatusResponse {
    pub fn to_str(&self) -> ~str {
        json::to_pretty_str(**self)
    }
}

pub struct TrainingProblem {
    challenge: ~str,
    id: ~str,
    size: u8,
    operators: OperatorSet,
}

impl WebEval for TrainingProblem {
    fn get_id(&self) -> ~str {
        self.id.to_owned()
    }
}

#[deriving(Clone, Eq)]
pub struct RealProblem {
    id: ~str,
    size: u8,
    operators: OperatorSet,
    time_left: Option<float>,
    solved: bool,
}

impl Ord for RealProblem {
    fn lt(&self, other: &RealProblem) -> bool {
        self.size < other.size
    }
}


impl WebEval for RealProblem {
    fn get_id(&self) -> ~str {
        self.id.to_owned()
    }
}

pub struct OperatorSet {
    op1: [bool, ..5],
    op2: [bool, ..4],
    if0: bool,
    fold: bool,
    tfold: bool,
}

impl OperatorSet {
    pub fn new() -> OperatorSet {
        OperatorSet {
            op1: [false, false, false, false, false],
            op2: [false, false, false, false],
            if0: false,
            fold: false,
            tfold: false,
        }
    }

    pub fn add(&mut self, ops: ~[~str]) {
        for op in ops.iter() {
            match *op {
                ~"not" => self.op1[OP_NOT] = true,
                ~"shl1" => self.op1[OP_SHL1] = true,
                ~"shr1" => self.op1[OP_SHR1] = true,
                ~"shr4" => self.op1[OP_SHR4] = true,
                ~"shr16" => self.op1[OP_SHR16] = true,
                ~"and" => self.op2[OP_AND] = true,
                ~"or" => self.op2[OP_OR] = true,
                ~"xor" => self.op2[OP_XOR] = true,
                ~"plus" => self.op2[OP_PLUS] = true,
                ~"if0" => self.if0 = true,
                ~"fold" => self.fold = true,
                ~"tfold" => self.tfold = true,
                _ => fail!("bad operation"),
            }
        }
    }
}

impl Clone for OperatorSet {
    pub fn clone(&self) -> OperatorSet {
        let mut ops = OperatorSet::new();
        for i in range(0, 5) {
            ops.op1[i] = self.op1[i];
            if i != 4 {
                ops.op2[i] = self.op2[i];
            }
        }
        ops.if0 = self.if0;
        ops.fold = self.fold;
        ops.tfold = self.tfold;
        ops
    }
}

impl Eq for OperatorSet {
    pub fn eq(&self, other: &OperatorSet) -> bool {
        for i in range(0, 5) {
            if other.op1[i] != self.op1[i] { return false; }
            if i != 4 {
                if other.op2[i] != self.op2[i] { return false; }
            }
        }
        if other.if0 != self.if0 { return false; }
        if other.fold != self.fold { return false; }
        if other.tfold != self.tfold { return false; }
        true
    }
}

pub enum GuessResult {
    Win,
    Mismatch(u64, u64, u64),
    Error(~str)
}

pub trait WebEval {
    fn get_id(&self) -> ~str;

    fn guess(&self, prog: ~str) -> GuessResult {
        let mut obj: TreeMap<~str, Json> = TreeMap::new();
        obj.insert(~"id", self.get_id().to_json());
        obj.insert(~"program", prog.to_json());
        let guess_json = obj.to_json().to_str();

        let response = match post_request(make_url("guess"), guess_json) {
            ~Object(o) => o,
            _ => fail!("bad guess response")
        };

        match get_json_str(response, ~"status") {
            ~"win" => Win,
            ~"mismatch" => fail!(),
            _ => fail!()
        }
    }
    fn eval(&self, nums: &[u64]) -> Option<~[u64]> {
        let mut obj: TreeMap<~str, Json> = TreeMap::new();
        obj.insert(~"id", self.get_id().to_json());

        let args = nums.iter().transform(|i| i.to_str_radix(16)).to_owned_vec();
        obj.insert(~"arguments", args.to_json());
        let eval_json = obj.to_json().to_str();

        let response = match post_request(make_url("eval"), eval_json) {
            ~Object(o) => o,
            _ => fail!("bad eval response")
        };

        if "ok" == get_json_str(response, ~"status") {
            let outs = get_json_array(response, ~"outputs");
            Some(do outs.iter().transform |j| {
                    match *j {
                        String(ref s) => {
                            let no_0x = s.slice_from(2);
                            FromStrRadix::from_str_radix::<u64>(no_0x, 16).expect("not a hex number")
                        }
                        _ => fail!("non string in eval.outputs")
                    }
                }.to_owned_vec())
        } else {
            println(get_json_str(response, ~"message"));
            None
        }
    }
}



fn get_json_array<'a>(obj: &'a Object, key: ~str) -> ~[Json] {
    match obj.find(&key) {
        Some(&List(ref l)) => l.clone(),
        _ => fail!(fmt!("unexpected type for '%s'", key)),
    }
}

fn get_json_num(obj: &Object, key: ~str) -> float {
    match obj.find(&key) {
        Some(&Number(n)) => n,
        _ => fail!(fmt!("unexpected type for '%s'", key)),
    }
}

fn get_json_str(obj: &Object, key: ~str) -> ~str {
    match obj.find(&key) {
        Some(&String(ref s)) => s.clone(),
        _ => fail!(fmt!("unexpected type for '%s'", key)),
    }
}

fn make_url(path: &str) -> ~str {
    let mut s = ~"http://icfpc2013.cloudapp.net/";
    s.push_str(path);
    s.push_str("?auth=");
    s.push_str(PRIVATE_KEY.trim());
    s.push_str("vpsH1H");
    s
}

fn get_request(url: ~str) -> ~Json {
    let mut p = Process::new("curl", [url], ProcessOptions::new());
    match json::from_reader(p.output()) {
        Ok(res) => ~res,
        Err(e) => fail!(e.to_str()),
    }
}

fn post_request(url: ~str, data: ~str) -> ~Json {
    let mut p = Process::new("curl", [~"-X", ~"POST", url, ~"-d", data], ProcessOptions::new());

    match json::from_reader(p.output()) {
        Ok(res) => ~res,
        Err(e) => fail!(e.to_str()),
    }
}

use program::*;

use std::cell::Cell;
use std::comm;
use std::io::ReaderUtil;
use std::run::{Process, ProcessOptions};
use std::rt::rtio::RtioTimer;
use std::rt::io::Timer;
use std::str;
use std::to_str::ToStr;
use std::num::{FromStrRadix,ToStrRadix};
use extra::json;
use extra::json::{Json, ToJson, Object, Number, String, List, Boolean};
use extra::time;
use extra::treemap::TreeMap;

static SERVER: &'static str = "http://icfpc2013.cloudapp.net/";

static PRIVATE_KEY: &'static str = include_str!("private.key");

pub struct WebApi(Chan<Request>);

impl WebApi {
    pub fn new() -> WebApi {
        let (port, chan) = comm::stream();

        let port = Cell::new(port);
        do spawn {
            WebApi::run(port.take());
        }

        WebApi(chan)
    }

    fn run(port: Port<Request>) {
        let mut running = true;
        let mut last_req = 0;
        let timer = Timer::new().unwrap();
        while running {
            match port.try_recv() {
                None => running = false,
                Some(req) => {
                    timer.sleep(3500);
                    dispatch(req);
                }
            }


        }
    }

    pub fn get_status(&mut self) -> Port<StatusResponse> {
        let (port, chan) = comm::stream();
        (**self).send(Status(chan));
        port
    }

    pub fn get_status_blocking(&mut self) -> StatusResponse {
        let port = self.get_status();
        port.recv()
    }

    pub fn get_problems(&mut self) -> Port<~[RealProblem]> {
        let (port, chan) = comm::stream();
        (**self).send(Problems(chan));
        port
    }

    pub fn get_problems_blocking(&mut self) -> ~[RealProblem] {
        let port = self.get_problems();
        port.recv()
    }

    pub fn eval(&mut self, problem: RealProblem, inputs: ~[u64]) -> Port<Option<~[u64]>> {
        let (port, chan) = comm::stream();
        (**self).send(Eval(problem, inputs, chan));
        port
    }

    pub fn eval_blocking(&mut self, problem: RealProblem, inputs: ~[u64]) -> Option<~[u64]> {
        let port = self.eval(problem, inputs);
        port.recv()
    }

    pub fn guess(&mut self, problem: RealProblem, program: ~str) -> Port<GuessResult> {
        let (port, chan) = comm::stream();
        (**self).send(Guess(problem, program, chan));
        port
    }

    pub fn guess_blocking(&mut self, problem: RealProblem, program: ~str) -> GuessResult {
        let port = self.guess(problem, program);
        port.recv()
    }
}

fn dispatch(req: Request) {
    match req {
        Status(ref resp_chan)  => {
            let response = get_request(req.to_url());
            resp_chan.send(StatusResponse(response));
        }
        Train(_, _, ref resp_chan) => {
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

                    resp_chan.send(TrainProblem {
                        challenge: challenge,
                        id: id,
                        size: size as u8,
                        operators: ops,
                    });
                }
                _ => fail!("bad response"),
            }
        }
        Problems(resp_chan) => {
            let response = match get_request(make_url("myproblems")) {
                ~List(a) => a,
                _ => fail!("bad myproblems response")
            };

            let probs = do response.consume_iter().transform |x| {
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
            }.collect();

            resp_chan.send(probs);
        }
        Eval(prob, inputs, resp_chan) => resp_chan.send(prob.eval(inputs)),
        Guess(prob, prog, resp_chan) => resp_chan.send(prob.guess(prog)),
    }
}

enum Request {
    Status(Chan<StatusResponse>),
    Train(u8, TrainOperator, Chan<TrainProblem>),
    Problems(Chan<~[RealProblem]>),
    Eval(RealProblem, ~[u64], Chan<Option<~[u64]>>),
    Guess(RealProblem, ~str, Chan<GuessResult>),
}

pub enum TrainOperator {
    Empty,
    Tfold,
    Fold,
}

impl Request {
    pub fn to_url(&self) -> ~str {
        match *self {
            Status(*) => {
                make_url("status")
            }
            Train(*) => {
                make_url("train")
            }
            _ => fail!("unsupported to_url kind"),
        }
    }

    pub fn to_json_str(&self) -> ~str {
        match *self {
            Train(size, ref ops, _) => {
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
}

struct StatusResponse(~Json);

impl ToStr for StatusResponse {
    pub fn to_str(&self) -> ~str {
        json::to_pretty_str(**self)
    }
}

pub struct TrainProblem {
    challenge: ~str,
    id: ~str,
    size: u8,
    operators: OperatorSet,
}

impl WebEval for TrainProblem {
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
    let output = str::from_bytes(p.output().read_whole_stream());
    match json::from_str(output) {
        Ok(res) => ~res,
        Err(e) => fail!(fmt!("error: %s\n%s", e.to_str(), output)),
    }
}

fn post_request(url: ~str, data: ~str) -> ~Json {
    let mut p = Process::new("curl", [~"-X", ~"POST", url.clone(), ~"-d", data], ProcessOptions::new());
    let output = str::from_bytes(p.output().read_whole_stream());
    match json::from_str(output) {
        Ok(res) => ~res,
        Err(e) => fail!(fmt!("error: %s\n%s", e.to_str(), output)),
    }
}

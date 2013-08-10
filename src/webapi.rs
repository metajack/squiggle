use program::*;

use std::cell::Cell;
use std::comm;
use std::hashmap::HashMap;
use std::io::ReaderUtil;
use std::rand;
use std::run::{Process, ProcessOptions};
use std::rt::rtio::RtioTimer;
use std::rt::io::timer::Timer;
use std::str;
use std::task;
use std::to_str::ToStr;
use std::num::{FromStrRadix,ToStrRadix};
use std::util;
use extra::json;
use extra::json::{Json, ToJson, Object, Number, String, List, Boolean};
use extra::time;
use extra::treemap::TreeMap;

static SERVER: &'static str = "http://icfpc2013.cloudapp.net/";

static PRIVATE_KEY: &'static str = include_str!("private.key");

pub struct WebApi(Chan<Request>);

pub trait Api {
    fn get_training(&mut self, size: u8, operator: TrainOperator) -> Port<TrainProblem>;
    fn get_training_blocking(&mut self, size: u8, operator: TrainOperator) -> TrainProblem {
        self.get_training(size, operator).recv()
    }

    fn get_problems(&mut self) -> Port<~[RealProblem]>;
    fn get_problems_blocking(&mut self) -> ~[RealProblem] {
        self.get_problems().recv()
    }

    fn eval(&mut self, problem: Problem, inputs: ~[u64]) -> Port<Option<~[u64]>>;
    fn eval_blocking(&mut self, problem: Problem, inputs: ~[u64]) -> Option<~[u64]> {
        self.eval(problem, inputs).recv()
    }

    fn guess(&mut self, problem: Problem, program: ~str) -> Port<GuessResult>;
    fn guess_blocking(&mut self, problem: Problem, program: ~str) -> GuessResult {
        self.guess(problem, program).recv()
    }
}

impl WebApi {
    pub fn new() -> WebApi {
        let (port, chan) = comm::stream();

        let port = Cell::new(port);
        do task::spawn_sched(task::SingleThreaded) {
            WebApi::run(port.take());
        }

        WebApi(chan)
    }

    fn run(port: Port<Request>) {
        let mut running = true;
        let timer = Timer::new().unwrap();

        let status = StatusResponse::from_json(get_request(make_url("status")));
        let mut req_win = status.request_window;
        let mut last_reset = time::precise_time_ns();

        while running {
            match port.try_recv() {
                None => running = false,
                Some(req) => {
                    if req_win.amount >= req_win.limit {
                        // over request limit, must wait until reset
                        let elapsed_ms = (time::precise_time_ns() - last_reset) / 1000000;
                        let ms_to_sleep = (1000f * req_win.resets_in) - (elapsed_ms as float);
                        if ms_to_sleep > 0f {
                            timer.sleep(ms_to_sleep as u64);
                            req_win.resets_in = 20f; // may change, but hardcoded for now
                            last_reset = time::precise_time_ns();
                        } else {
                            // we are past the window, ms_to_sleep is negative
                            req_win.resets_in = 20f + ms_to_sleep; // may change, but hardcoded for now
                            last_reset = time::precise_time_ns() + (ms_to_sleep as u64) * 1000000;
                        }
                        req_win.amount = 0f;
                    }

                    dispatch(req);

                    req_win.amount += 1f;
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
}
impl Api for WebApi {
    pub fn get_training(&mut self, size: u8, operator: TrainOperator) -> Port<TrainProblem> {
        let (port, chan) = comm::stream();
        (**self).send(Train(size, operator, chan));
        port
    }

    pub fn get_problems(&mut self) -> Port<~[RealProblem]> {
        let (port, chan) = comm::stream();
        (**self).send(Problems(chan));
        port
    }

    pub fn eval(&mut self, problem: Problem, inputs: ~[u64]) -> Port<Option<~[u64]>> {
        let (port, chan) = comm::stream();
        (**self).send(Eval(problem, inputs, chan));
        port
    }

    pub fn guess(&mut self, problem: Problem, program: ~str) -> Port<GuessResult> {
        let (port, chan) = comm::stream();
        (**self).send(Guess(problem, program, chan));
        port
    }
}

pub struct FakeApi {
    programs: ~[Program],
    // for checking answers
    by_str: HashMap<~str, Program>
}

impl FakeApi {
    pub fn new(progs: ~[Program]) -> FakeApi {
        // this copies every program, but it shouldn't matter much at
        // all: only run once.
        let by_str = progs.iter().transform(|p| (p.to_str(), p.clone())).collect();
        FakeApi {
            programs: progs,
            by_str: by_str
        }
    }

    fn get_prog<'a>(&'a self, id: &str) -> &'a Program {
        self.by_str.find_equiv(&id).expect(fmt!("unknown problem id %s", id))
    }

    pub fn has_programs(&self) -> bool {
        !self.programs.is_empty()
    }
}

impl Api for FakeApi {
    pub fn get_training(&mut self, _size: u8, _operator: TrainOperator) -> Port<TrainProblem> {
        let prog = self.programs.shift();
        let prog_str = prog.to_str();

        let tp = TrainProblem {
            challenge: prog_str.clone(),
            problem: Problem {
                id: prog_str,
                size: prog.len(),
                operators: prog.operators()
            }
        };

        let (port, chan) = comm::stream();
        chan.send(tp);
        port
    }

    pub fn get_problems(&mut self) -> Port<~[RealProblem]> {
        let mut progs = ~[];
        util::swap(&mut progs, &mut self.programs);

        let real_probs = do progs.consume_iter().transform |p| {
            RealProblem {
                problem: Problem {
                    id: p.to_str(),
                    size: p.len(),
                    operators: p.operators()
                },
                time_left: None,
                solved: false
            }
        }.collect();

        let (port, chan) = comm::stream();
        chan.send(real_probs);
        port
    }

    pub fn eval(&mut self, problem: Problem, inputs: ~[u64]) -> Port<Option<~[u64]>> {
        let prog = self.get_prog(problem.id);

        let outs = do inputs.consume_iter().transform |x| {
            prog.eval(x)
        }.collect();

        let (port, chan) = comm::stream();
        chan.send(Some(outs));
        port
    }

    pub fn guess(&mut self, problem: Problem, program: ~str) -> Port<GuessResult> {
        use compile::compile_program;
        use parse::Parse;

        let parsed = compile_program(&program.parse());
        let real = compile_program(self.get_prog(problem.id));
        let mut rng = rand::task_rng();

        let mut result = Win;
        for _ in range(0, 100000) {
            let x = rng.gen();
            let test = parsed.eval(x);
            let expected = real.eval(x);
            if test != expected {
                result = Mismatch(x, expected, test);
                break
            }
        }

        let (port, chan) = comm::stream();
        chan.send(result);
        port
    }
}

fn dispatch(req: Request) {
    match req {
        Status(ref resp_chan)  => {
            let response = get_request(make_url("status"));
            resp_chan.send(StatusResponse::from_json(response));
        }
        Train(_, _, ref resp_chan) => {
            let response = post_request(make_url("train"), req.to_json_str());

            match response {
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
                        problem: Problem {
                            id: id,
                            size: size as u8,
                            operators: ops,
                        },
                    });
                }
                _ => fail!("bad response"),
            }
        }
        Problems(resp_chan) => {
            let response = match get_request(make_url("myproblems")) {
                List(a) => a,
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
                            problem: Problem {
                                id: id,
                                size: size as u8,
                                operators: ops,
                            },
                            time_left: time_left,
                            solved: solved,
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
    Eval(Problem, ~[u64], Chan<Option<~[u64]>>),
    Guess(Problem, ~str, Chan<GuessResult>),
}

pub enum TrainOperator {
    Empty,
    Tfold,
    Fold,
}

impl Request {
    pub fn to_json_str(&self) -> ~str {
        match *self {
            Train(size, ref ops, _) => {
                let mut obj: TreeMap<~str, Json> = TreeMap::new();
                obj.insert(~"size", size.to_json());
                obj.insert(~"operators", match *ops {
                    Empty => List(~[]),
                    Tfold => (~[~"tfold"]).to_json(),
                    Fold => (~[~"fold"]).to_json(),
                });
                obj.to_json().to_str()
            }
            _ => fail!(~"not implemented"),
        }
    }
}

struct StatusResponse {
    easy_chair_id: ~str,
    contest_score: float,
    lightning_score: float,
    training_score: float,
    mismatches: float,
    num_requests: float,
    cpu_total_time: float,
    request_window: Window,
    cpu_window: Window,
}

struct Window {
    resets_in: float,
    amount: float,
    limit: float,
}

impl Window {
    pub fn from_json(data: Json) -> Window {
        match data {
            Object(ref obj) => {
                let resets_in = get_json_num(*obj, ~"resetsIn");
                let amount = get_json_num(*obj, ~"amount");
                let limit = get_json_num(*obj, ~"limit");

                Window {
                    resets_in: resets_in,
                    amount: amount,
                    limit: limit,
                }
            }
            _ => fail!("unexpected data"),
        }
    }
}

impl StatusResponse {
    pub fn from_json(data: Json) -> StatusResponse {
        match data {
            Object(ref obj) => {
                let easy_chair_id = get_json_str(*obj, ~"easyChairId");
                let contest_score = get_json_num(*obj, ~"contestScore");
                let lightning_score = get_json_num(*obj, ~"lightningScore");
                let training_score = get_json_num(*obj, ~"trainingScore");
                let mismatches = get_json_num(*obj, ~"mismatches");
                let num_requests = get_json_num(*obj, ~"numRequests");
                let cpu_total_time = get_json_num(*obj, ~"cpuTotalTime");

                let request_window = match obj.find(&~"requestWindow") {
                    Some(win_data) => Window::from_json(win_data.clone()),
                    _ => fail!("bad requestWindow"),
                };
                let cpu_window = match obj.find(&~"cpuWindow") {
                    Some(win_data) => Window::from_json(win_data.clone()),
                    _ => fail!("bad cpuWindow"),
                };

                StatusResponse {
                    easy_chair_id: easy_chair_id,
                    contest_score: contest_score,
                    lightning_score: lightning_score,
                    training_score: training_score,
                    mismatches: mismatches,
                    num_requests: num_requests,
                    cpu_total_time: cpu_total_time,
                    request_window: request_window,
                    cpu_window: cpu_window,
                }
            }
            _ => fail!("unexpected data"),
        }
    }

    pub fn score_report(&self) {
        printfln!("contest: %u \t lightning: %u \t training: %u",
                  self.contest_score as uint,
                  self.lightning_score as uint,
                  self.training_score as uint);
    }
}

#[deriving(Clone, Eq)]
pub struct Problem {
    id: ~str,
    size: u8,
    operators: OperatorSet,
}

pub struct TrainProblem {
    challenge: ~str,
    problem: Problem,
}

#[deriving(Clone, Eq)]
pub struct RealProblem {
    problem: Problem,
    time_left: Option<float>,
    solved: bool,
}

impl Ord for RealProblem {
    fn lt(&self, other: &RealProblem) -> bool {
        self.problem.size < other.problem.size
    }
}


impl WebEval for Problem {
    fn get_id(&self) -> ~str {
        self.id.to_owned()
    }
}

pub enum GuessResult {
    Win,
    Mismatch(u64, u64, u64),
    Error(~str)
}

trait WebEval {
    fn get_id(&self) -> ~str;

    fn guess(&self, prog: ~str) -> GuessResult {
        let mut obj: TreeMap<~str, Json> = TreeMap::new();
        obj.insert(~"id", self.get_id().to_json());
        obj.insert(~"program", prog.to_json());
        let guess_json = obj.to_json().to_str();

        let response = match post_request(make_url("guess"), guess_json) {
            Object(o) => o,
            _ => fail!("bad guess response")
        };

        match get_json_str(response, ~"status") {
            ~"win" => Win,
            ~"mismatch" => {
                let vals = get_json_array(response, ~"values");
                assert!(vals.len() == 3, "not enough values in guess.mismatch");
                let mut parsed = do vals.iter().transform |s| {
                    read_0x_hex(unwrap_json_str(s))
                };
                Mismatch(parsed.next().unwrap(),
                         parsed.next().unwrap(),
                         parsed.next().unwrap())
            }
            _ => {
                Error(get_json_str(response, ~"message"))
            }
        }
    }
    fn eval(&self, nums: &[u64]) -> Option<~[u64]> {
        let mut obj: TreeMap<~str, Json> = TreeMap::new();
        obj.insert(~"id", self.get_id().to_json());

        let args = nums.iter().transform(|i| i.to_str_radix(16)).to_owned_vec();
        obj.insert(~"arguments", args.to_json());
        let eval_json = obj.to_json().to_str();

        let response = match post_request(make_url("eval"), eval_json) {
            Object(o) => o,
            _ => fail!("bad eval response")
        };

        if "ok" == get_json_str(response, ~"status") {
            let outs = get_json_array(response, ~"outputs");
            Some(do outs.iter().transform |j| {
                    read_0x_hex(unwrap_json_str(j))
                }.to_owned_vec())
        } else {
            println(get_json_str(response, ~"message"));
            None
        }
    }
}

fn read_0x_hex(s: &str) -> u64 {
    let no_0x = s.slice_from(2);
    FromStrRadix::from_str_radix::<u64>(no_0x, 16).expect("not a hex number")
}

fn unwrap_json_str<'a>(json: &'a Json) -> &'a str {
    match *json {
        String(ref s) => s.as_slice(),
        _ => fail!("non string")
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

fn get_request(url: ~str) -> Json {
    let mut tries = 5;
    while tries > 0 {
        info!("GET /%s", extract_path(url));
        let mut p = Process::new("curl", [~"-f", url.clone()], ProcessOptions::new());
        let output = str::from_bytes(p.output().read_whole_stream());
        let retval = p.finish();
        info!("HTTP %i: %s", retval, output);
        if retval == 0 {
            match json::from_str(output) {
                Ok(res) => return res,
                Err(e) => fail!(fmt!("error: %s\n%s", e.to_str(), output)),
            }
        } else {
            println("WARN: http throttled. retrying");
            tries -= 1;
            let timer = Timer::new().unwrap();
            timer.sleep(4000);
        }
    }
    fail!("ran out of retries");
}

fn post_request(url: ~str, data: ~str) -> Json {
    let mut tries = 5;
    while tries > 0 {
        info!("POST /%s", extract_path(url));
        info!("DATA: %s", data);
        let mut p = Process::new("curl", [~"-X", ~"POST", ~"-f", url.clone(), ~"-d", data.clone()], ProcessOptions::new());
        let output = str::from_bytes(p.output().read_whole_stream());
        let retval = p.finish();
        info!("HTTP %i: %s", retval, output);
        if retval == 0 {
            match json::from_str(output) {
                Ok(res) => return res,
                Err(e) => fail!(fmt!("error: %s\n%s", e.to_str(), output)),
            }
        } else {
            println("WARN: http throttled. retrying");
            tries -= 1;
            let timer = Timer::new().unwrap();
            timer.sleep(4000);
        }
    }
    fail!("ran out of retries");
}

fn extract_path(url: &str) -> ~str {
    let mut pieces = url.split_iter('/');
    let path_opt = pieces.nth(3);
    let path = path_opt.get_ref().to_owned();
    let mut pieces = path.split_iter('?');
    let path = pieces.nth(0);
    path.get_ref().to_owned()
}

use std::hashmap::HashSet;
use std::run::{Process, ProcessOptions};
use std::to_str::ToStr;
use extra::json;
use extra::json::{Json, ToJson, Object, Number, String, List};
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

    pub fn get_eval_results(_tests: ~[u64]) -> ~[(u64, u64)] {
        // TODO: implement this
        ~[]
    }

    pub fn get_training_problem(size: u8, operators: TrainOperators) -> TrainingProblem {
        let req = Train { size: size, operators: operators };
        let response = post_request(req.to_url(), req.to_json_str());

        match *response {
            Object(obj) => {
                let challenge = get_json_str(obj, ~"challenge");
                let id = get_json_str(obj, ~"id");
                let size = get_json_num(obj, ~"size");
                let mut ops = ~HashSet::new();
                let array = get_json_array(obj, ~"operators");
                for op in array.iter() {
                    match *op {
                        String(ref s) => ops.insert(s.clone()),
                        _ => fail!("bad value in 'operators'"),
                    };
                };

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
    operators: ~HashSet<~str>,
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

use std::run::{Process, ProcessOptions};
use std::to_str::ToStr;
use extra::json;
use extra::json::{Json, Object, ToJson};
use extra::treemap::TreeMap;
use extra::url;

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

    pub fn get_training_problem(size: u8, operators: TrainOperators) {
        let req = Train { size: size, operators: operators };
        let response = post_request(req.to_url(), req.to_json_str());
        println(response.to_str());
    }
}

struct StatusResponse(~Json);

impl ToStr for StatusResponse {
    pub fn to_str(&self) -> ~str {
        json::to_pretty_str(**self)
    }
}

struct TrainingProblem {
    challenge: ~str,
    id: ~str,
    size: u8,
    operators: ~[~str],
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

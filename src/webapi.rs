use std::run::{Process, ProcessOptions};
use std::to_str::ToStr;
use extra::json;
use extra::json::Json;

static SERVER: &'static str = "http://icfpc2013.cloudapp.net/";

static PRIVATE_KEY: &'static str = include_str!("private.key");

enum Request {
    Status,
    Train { size: u8, operators: ~str },
}

impl Request {
    pub fn to_url(&self) -> ~str {
        match *self {
            Status => {
                make_url("status")
            }
            _ => fail!(~"not implemented"),
        }
    }

    pub fn status() -> StatusResponse {
        let response = get_request(Status.to_url());
        StatusResponse(response)
    }
}

struct StatusResponse(~Json);

impl ToStr for StatusResponse {
    pub fn to_str(&self) -> ~str {
        json::to_pretty_str(**self)
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
    ~json::from_reader(p.output()).unwrap()
}

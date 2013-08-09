use std::run::{Process, ProcessOptions};
use extra::json;

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

    pub fn status() {
        println(Status.to_url());
        let mut p = Process::new(
            "curl",
            [Status.to_url()],
            ProcessOptions::new());

        let out = p.output();
        println(fmt!("read: %s", 
                     json::to_pretty_str(&json::from_reader(out).unwrap())));
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

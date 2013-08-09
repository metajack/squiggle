use extra::url::Url;

static SERVER: &'static str = "http://icfpc2013.cloudapp.net/";

static PRIVATE_KEY: &'static str = include_str!("private.key");

enum Request {
    Status,
    Train { size: u8, operators: ~str },
}

impl Request {
    pub fn to_url(&self) -> Url {
        match self {
            Status => {
                make_url("status")
            }
            _ => fail!(~"not implemented"),
        }
    }

}

priv fn make_url(path: &str) -> Url {
    let key = PRIVATE_KEY.clone().push_str("vpsH1H");
    Url::new(
        ~"http",
        None,
        ~"icfpc2013.cloudapp.net",
        None,
        path.clone(),
        ~[(~"auth", key)])
}

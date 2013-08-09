use std::num::ToStrRadix;
use std::rand::{Rng, XorShiftRng};


type Scope = ~[~str];

struct ScopeStack {
    stack: ~[Scope],
}

struct NaiveGen {
    rng: ~XorShiftRng,
    scopes: ScopeStack,
    next_symbol: u8,
}

impl NaiveGen {
    pub fn gensym(&mut self) -> ~str {
        let mut num = self.next_symbol;
        self.next_symbol += 1;

        num.to_str_radix(26)
    }
}

impl Rng for NaiveGen {
    pub fn next(&mut self) -> u32 {
        self.rng.next()
    }
}
use program::*;

use std::num::ToStrRadix;
use std::rand::{Rng, RngUtil, XorShiftRng};

pub trait Generator {
    pub fn gen_sym(&mut self) -> ~str;
    pub fn gen_expr(&mut self) -> Expr;
    pub fn gen_prog(&mut self) -> Program;

    pub fn get_sym(&mut self) -> ~str;
}

type Scope = ~[~str];

struct ScopeStack {
    stack: ~[Scope],
}

pub struct NaiveGen {
    rng: XorShiftRng,
    scopes: ScopeStack,
    next_symbol: u8,
}

impl NaiveGen {
    pub fn new() -> NaiveGen {
        NaiveGen {
            rng: XorShiftRng::new(),
            scopes: ScopeStack {
                stack: ~[],
            },
            next_symbol: 0,
        }
    }

    pub fn reset(&mut self) {
        self.scopes = ScopeStack {
            stack: ~[],
        };
        self.next_symbol = 0;
    }
}

impl Generator for NaiveGen {
    pub fn gen_sym(&mut self) -> ~str {
        let mut num = self.next_symbol;
        self.next_symbol += 1;

        num.to_str_radix(26)
    }

    pub fn gen_expr(&mut self) -> Expr {
        let choice = self.rng.gen_uint_range(0, 6);
        match choice {
            0 => Zero,
            1 => One,
            2 => Ident(self.get_sym()),
            3 => {
                let test = self.gen_expr();
                let then = self.gen_expr();
                let other = self.gen_expr();
                If0(~test, ~then, ~other)
            }
            4 => {
                let op = self.rng.gen();
                let expr = self.gen_expr();
                Op1(op, ~expr)
            }
            5 => {
                let op = self.rng.gen();
                let left = self.gen_expr();
                let right = self.gen_expr();
                Op2(op, ~left, ~right)
            }
            _ => fail!(~"unexpected random value"),
        }
    }

    pub fn gen_prog(&mut self) -> Program {
        let sym = self.gen_sym();
        let scope = ~[sym.clone()];
        self.scopes.stack.push(scope);
        let expr = self.gen_expr();
        Program::new(sym, ~expr)
    }

    pub fn get_sym(&mut self) -> ~str {
        let syms = self.scopes.stack.concat_vec();
        self.rng.choose(syms)
    }
}

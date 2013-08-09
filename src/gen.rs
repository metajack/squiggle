use program::*;

use std::num::ToStrRadix;
use std::rand::{Rng, RngUtil, IsaacRng, IsaacRng};
use std::str;

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
    rng: IsaacRng,
    scopes: ScopeStack,
    next_symbol: u8,
    max_size: u8,
    size: u8,
}

impl NaiveGen {
    pub fn new(max_size: u8) -> NaiveGen {
        assert!(max_size >= 3 && max_size <= 30);
        NaiveGen {
            rng: IsaacRng::new(),
            scopes: ScopeStack {
                stack: ~[],
            },
            next_symbol: 0,
            max_size: max_size,
            size: 0,
        }
    }

    pub fn reset(&mut self) {
        self.scopes = ScopeStack {
            stack: ~[],
        };
        self.next_symbol = 0;
        self.size = 0;
    }
}

impl Generator for NaiveGen {
    pub fn gen_sym(&mut self) -> ~str {
        let mut num = self.next_symbol;
        self.next_symbol += 1;

        let mut div = 0;
        let mut rem = 0;
        let mut id = ~[];
        loop {
            let (div0, rem0) = num.div_rem(&26);
            div = div0;
            rem = rem0;
            id.push(rem + 97 as u8);
            if div == 0 {
                break;
            }
        }
        str::from_bytes(id)
    }

    pub fn gen_expr(&mut self) -> Expr {
        loop {
            let choice = self.rng.gen_uint_range(0, 7);
            match choice {
                0 => {
                    self.size += 1;
                    return Zero;
                }
                1 => {
                    self.size += 1;
                    return One;
                }
                2 => {
                    self.size += 1;
                    return Ident(self.get_sym());
                }
                3 => {
                    if self.size + 4 <= self.max_size {
                        self.size += 1;
                        let test = self.gen_expr();
                        let then = self.gen_expr();
                        let other = self.gen_expr();
                        return If0(~test, ~then, ~other);
                    }
                    loop;
                }
                4 => {
                    if self.size + 2 <= self.max_size {
                        self.size += 1;
                        let op = self.rng.gen();
                        let expr = self.gen_expr();
                        return Op1(op, ~expr);
                    }
                    loop;
                }
                5 => {
                    if self.size + 3 <= self.max_size {
                        let op = self.rng.gen();
                        let left = self.gen_expr();
                        let right = self.gen_expr();
                        return Op2(op, ~left, ~right);
                    }
                    loop;
                }
                6 => {
                    if self.size + 5 <= self.max_size {
                        let foldee = self.gen_expr();
                        let init = self.gen_expr();
                        let next_id = self.gen_sym();
                        let accum_id = self.gen_sym();
                        let scope = ~[next_id.clone(), accum_id.clone()];
                        self.scopes.stack.push(scope);
                        let body = self.gen_expr();
                        self.scopes.stack.pop();
                        return Fold {
                            foldee: ~foldee,
                            init: ~init,
                            next_id: next_id,
                            accum_id: accum_id,
                            body: ~body,
                        };
                    }
                    loop;
                }
                _ => fail!(~"unexpected random value"),
            }
        }
    }

    pub fn gen_prog(&mut self) -> Program {
        let sym = self.gen_sym();
        let scope = ~[sym.clone()];
        self.scopes.stack.push(scope);
        self.size += 1;
        let expr = self.gen_expr();
        Program::new(sym, ~expr)
    }

    pub fn get_sym(&mut self) -> ~str {
        let syms = self.scopes.stack.concat_vec();
        self.rng.choose(syms)
    }
}

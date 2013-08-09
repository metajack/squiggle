use program::*;

// immutable linked list, lifetime don't work well enough (yet; need
// the ability for 2 lifetimes) for this to avoid @.
struct Scope {
    id: ~str,
    val: u64,
    parent: Option<@Scope>
}

impl Scope {
    fn push(@self, id: ~str, val: u64) -> @Scope {
        @Scope {
            id: id,
            val: val,
            parent: Some(self)
        }
    }

    fn lookup(&self, id: &str) -> u64 {
        if id == self.id {
            self.val
        } else {
            match self.parent {
                Some(p) => p.lookup(id),
                None => fail!("ident %s not found", id)
            }
        }
    }

    fn eval(@self, expr: &Expr) -> u64 {
        match *expr {
            Zero => 0,
            One => 1,
            Ident(ref s) => self.lookup(*s),
            If0(~ref cond, ~ref then, ~ref els) => {
                if self.eval(cond) == 0 {
                    self.eval(then)
                } else {
                    self.eval(els)
                }
            }
            Op1(op, ~ref expr) => {
                let e = self.eval(expr);

                match op {
                    Not => !e,
                    Shl1 => e << 1,
                    Shr1 => e >> 1,
                    Shr4 => e >> 4,
                    Shr16 => e >> 16
                }
            }
            Op2(op, ~ref e1, ~ref e2) => {
                let e1 = self.eval(e1);
                let e2 = self.eval(e2);

                match op {
                    And => e1 & e2,
                    Or => e1 | e2,
                    Xor => e1 ^ e2,
                    Plus => e1 + e2
                }
            }
            Fold {
                foldee: ~ref foldee, init: ~ref init,
                next_id: ref next_id, accum_id: ref accum_id,
                body: ~ref body
            } => {
                let mut foldee = self.eval(foldee);
                let mut accum = self.eval(init);

                for _ in range(0, 8) {
                    let b = foldee & 0xff;
                    foldee >>= 8;

                    let scope = self.push(next_id.clone(), b);
                    let scope = scope.push(accum_id.clone(), accum);

                    accum = scope.eval(body)
                }

                accum
            }
        }
    }
}

pub trait Eval {
    fn eval(&self, val: u64) -> u64;
}
impl Eval for Program {
    fn eval(&self, val: u64) -> u64 {
        (@Scope {
                id: self.id.clone(),
                val: val,
                parent: None
            }).eval(self.expr)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use program::*;
    use extra::test::BenchHarness;
    use std::rand;

    #[bench]
    fn bench_eval(bh: &mut BenchHarness) {
        // collection of randomly generated programs
        let progs = [
                     Program::new(~"gg", ~Op1(Shr1, ~Ident(~"gg"))),
                     Program::new(~"hg", ~Op1(Shl1, ~Ident(~"hg"))),
                     ];
        let mut rng = rand::rng();

        do bh.iter {
            for p in progs.iter() {
                let p: &Program = p;
                for _ in range(0, 10) {
                    p.eval(rng.gen());
                }
            }
        }
    }
}

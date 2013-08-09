use program::*;
use eval::Eval;
use extra::smallintmap::SmallIntMap;

type Compiled = ~fn(&mut SmallIntMap<u64>) -> u64;

// type inference doesn't work perfectly for return types.
#[inline(always)]
fn c(f: Compiled) -> Compiled {
    f
}

fn compile(expr: &Expr) -> Compiled {
    match *expr {
        Zero => c(|_| 0),
        One => c(|_| 1),
        Ident(id) => {
            do c |map| {
                *map.find(&id).unwrap()
            }
        }
        Op1(op, ~ref expr) => {
            let expr_f = compile(expr);
            match op {
                Not => c(|m| !expr_f(m)),
                Shl1 => c(|m| expr_f(m) << 1),
                Shr1 => c(|m| expr_f(m) >> 1),
                Shr4 => c(|m| expr_f(m) >> 4),
                Shr16 => c(|m| expr_f(m) >> 16),
            }
        }
        Op2(op, ~ref e1, ~ref e2) => {
            let e1_f = compile(e1);
            let e2_f = compile(e2);

            match op {
                And => c(|m| e1_f(m) & e2_f(m)),
                Or => c(|m| e1_f(m) | e2_f(m)),
                Xor => c(|m| e1_f(m) ^ e2_f(m)),
                Plus => c(|m| e1_f(m) + e2_f(m))
            }
        }
        If0(~ref cond, ~ref then, ~ref els) => {
            let cond_f = compile(cond);
            let then_f = compile(then);
            let els_f = compile(els);

            do c |m| {
                if cond_f(m) == 0 {
                    then_f(m)
                } else {
                    els_f(m)
                }
            }
        }
        Fold {
            foldee: ~ref foldee, init: ~ref init,
            next_id, accum_id,
            body: ~ref body
        } => {
            let foldee_f = compile(foldee);
            let accum_f = compile(init);
            let body_f = compile(body);

            do c |m| {
                let mut foldee = foldee_f(m);
                let mut accum = accum_f(m);

                for _ in range(0, 8) {
                    let b = foldee & 0xff;
                    foldee >>= 8;

                    m.insert(next_id.clone(), b);
                    m.insert(accum_id.clone(), accum);

                    accum = body_f(m);
                }
                accum
            }
        }
    }
}

pub fn compile_program(p: &Program) -> CompiledProgram {
    CompiledProgram {
        id: p.id,
        expr: compile(p.expr)
    }
}

pub struct CompiledProgram {
    id: Id,
    expr: Compiled
}


impl Eval for CompiledProgram {
    fn eval(&self, val: u64) -> u64 {
        //let mut map = HashMap::with_capacity_and_keys(10, 234, 10);
        let mut map = SmallIntMap::new();
        map.insert(self.id, val);
        (self.expr)(&mut map)
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
                     compile_program(&Program::new(0, ~Op1(Shr1, ~Ident(0)))),
                     compile_program(&Program::new(0, ~Op1(Shl1, ~Ident(0)))),
                     ];
        let mut rng = rand::rng();

        do bh.iter {
            for p in progs.iter() {
                for _ in range(0, 10) {
                    p.eval(rng.gen());
                }
            }
        }
    }
    #[bench]
    fn bench_eval_fold(bh: &mut BenchHarness) {
        let fold_expr = ~Fold {
            foldee: ~Ident(0),
            init: ~Zero,
            next_id: 1,
            accum_id: 2,
            body: ~Op2(Plus, ~Ident(1), ~Ident(2))
        };
        let prog = compile_program(&Program::new(0, fold_expr));
        let mut rng = rand::rng();

        do bh.iter {
            prog.eval(rng.gen());
        }
    }

    #[test]
    fn test_eval() {
        let fold_expr = ~Fold {
            foldee: ~Ident(0),
            init: ~Zero,
            next_id: 1,
            accum_id: 2,
            body: ~Op2(Plus, ~Ident(1), ~Ident(2))
        };
        let fold_fn = |mut x: u64| {
            let mut accum = 0;
            for _ in range(0, 8) {
                accum += x & 0xff;
                x >>= 8;
            }
            accum
        };

        let progs_fn: ~[(Program, &fn(u64) -> u64)] = ~[
                        (Program::new(0, ~Zero), |_| 0),
                        (Program::new(0, ~One), |_| 1),
                        (Program::new(0, ~Ident(0)), |x| x),
                        (Program::new(0, ~Op1(Not, ~Ident(0))), |x| !x),
                        (Program::new(0, ~Op1(Shl1, ~Ident(0))), |x| x << 1),
                        (Program::new(0, ~Op1(Shr1, ~Ident(0))), |x| x >> 1),
                        (Program::new(0, ~Op1(Shr4, ~Ident(0))), |x| x >> 4),
                        (Program::new(0, ~Op1(Shr16, ~Ident(0))), |x| x >> 16),
                        (Program::new(0, ~Op2(And, ~Ident(0), ~One)), |x| x & 1),
                        (Program::new(0, ~Op2(Or, ~Ident(0), ~One)), |x| x | 1),
                        (Program::new(0, ~Op2(Xor, ~Ident(0), ~One)), |x| x ^ 1),
                        (Program::new(0, ~Op2(Plus, ~Ident(0), ~One)), |x| x + 1),
                        (Program::new(0, ~If0(~Ident(0), ~One, ~Zero)),
                         |x| if x == 0 {1} else {0}),
                        (Program::new(0, ~If0(~Zero, ~One, ~Zero)), |_| 1),
                        (Program::new(0, ~If0(~One, ~One, ~Zero)), |_| 0),
                        (Program::new(0, fold_expr), fold_fn),
                        ];

        for (p, f) in progs_fn.consume_iter() {
            info!(p.to_str());
            let compiled = compile_program(&p);

            // some "special cases", maybe.
            assert_eq!(compiled.eval(0), f(0));
            assert_eq!(compiled.eval(1), f(1));
            assert_eq!(compiled.eval(-1), f(-1));

            for _ in range(0, 100) {
                let i: u64 = rand::random();
                assert_eq!(compiled.eval(i), f(i));
            }
        }
    }
}

use webapi::*;
use program::*;

use std::cell::Cell;
use std::comm;
use std::comm::{Port, Chan};
use std::rand::{Rng, RngUtil, XorShiftRng, task_rng};
use std::task;

pub enum GenMsg {
    Generate(Chan<~Program>),
    Reset(Problem, ~[(u64, u64)]),
    MoreConstraints(~[(u64, u64)]),
    Exit,
}

pub struct RandomGen(Chan<GenMsg>);

impl RandomGen {
    pub fn new(problem: Problem, constraints: ~[(u64, u64)]) -> RandomGen {
        let (port, chan) = comm::stream();

        let port = Cell::new(port);
        do task::spawn_sched(task::SingleThreaded) {
            RandomGen::generate(problem.clone(), constraints.clone(), port.take());
        }

        RandomGen(chan)
    }

    pub fn blank() -> RandomGen {
        RandomGen::new(
            Problem {
                size: 3,
                operators: OperatorSet::new(),
                id: ~"",
            },
            ~[])
    }

    pub fn reset(&mut self, problem: Problem, constraints: ~[(u64, u64)]) {
        (**self).send(Reset(problem, constraints));
    }

    pub fn next(&mut self) -> ~Program {
        let (port, chan) = comm::stream();
        (**self).send(Generate(chan));
        port.recv()
    }

    pub fn more_constraints(&mut self, cs: ~[(u64, u64)]) {
        (**self).send(MoreConstraints(cs));
    }

    fn generate(mut problem: Problem, mut constraints: ~[(u64, u64)], port: Port<GenMsg>) {
        let mut gen = RandomGenState::new(problem.clone());
        loop {
            match port.try_recv() {
                None => break,
                Some(Exit) => break,
                Some(Reset(p, c)) => {
                    constraints = c;
                    gen.reset(p.clone());
                    problem = p;
                }
                Some(MoreConstraints(c)) => {
                    constraints.push_all_move(c)
                }
                Some(Generate(chan)) => {
                    let mut i = 0;
                    'newprog: loop {
                        let prog = gen.gen_program(problem.size as uint);
                        i += 1;
                        for &(x, y) in constraints.iter() {
                            if prog.eval(x) != y {
                                if i % 1000000 == 0 {
                                    printfln!("gen stats: searched for %u iters", i);
                                }
                                loop 'newprog;
                            }
                        }
                        if i > 1 {
                            printfln!("gen stats: candidate took %u iters", i);
                        }
                        chan.send(~prog);
                        break;
                    }
                }
            }
        }
    }
}

struct RandomGenState {
    rng: XorShiftRng,
    operators: OperatorSet,
    op1_len: uint,
    op1_choices: ~[UnaOp],
    op2_len: uint,
    op2_choices: ~[BinOp],
}

impl RandomGenState {
    fn new(problem: Problem) -> RandomGenState {
        let mut seed_rng = task_rng();
        let rng = XorShiftRng::new_seeded(
            seed_rng.gen::<u32>(),
            seed_rng.gen::<u32>(),
            seed_rng.gen::<u32>(),
            seed_rng.gen::<u32>());

        let op1_choices: ~[UnaOp] = (~[Not, Shl1, Shr1, Shr4, Shr16]).consume_iter()
            .filter(|o| o.in_ops(&problem.operators))
            .collect();
        let op2_choices: ~[BinOp] = (~[And, Or, Xor, Plus]).consume_iter()
            .filter(|o| o.in_ops(&problem.operators))
            .collect();

        RandomGenState {
            rng: rng,
            operators: problem.operators,
            op1_len: op1_choices.len(),
            op1_choices: op1_choices,
            op2_len: op2_choices.len(),
            op2_choices: op2_choices,
        }
    }

    fn reset(&mut self, problem: Problem) {
        let op1_choices: ~[UnaOp] = (~[Not, Shl1, Shr1, Shr4, Shr16]).consume_iter()
            .filter(|o| o.in_ops(&problem.operators))
            .collect();
        let op2_choices: ~[BinOp] = (~[And, Or, Xor, Plus]).consume_iter()
            .filter(|o| o.in_ops(&problem.operators))
            .collect();

        self.operators = problem.operators;
        self.op1_len = op1_choices.len();
        self.op1_choices = op1_choices;
        self.op2_len = op2_choices.len();
        self.op2_choices = op2_choices;
    }

    fn gen_program(&mut self, mut size: uint) -> Program {
        // remove the size of each program (1)
        size -= 1;

        let expr = if self.operators.tfold {
            // remove the sizes of fold, x and 0
            let body = ~self.gen_expr(size - 2 - 1 - 1, 2, false, false);
            Fold {
                foldee: ~Ident(0),
                init: ~Zero,
                accum_id: 0,
                next_id: 1,
                body: body
            }
        } else {
            self.gen_expr(size, 1, self.operators.fold, true)
        };
        Program::new(0, ~expr)
    }

    fn gen_expr(&mut self, size: uint, idents: uint, foldable: bool, root: bool) -> Expr {
        match size {
            1 => {
                // Choices:
                // Zero (1)
                // One (1)
                // Ident (idents)
                let choice = self.rng.gen_uint_range(0, 2 + idents);
                match choice {
                    0 => Zero,
                    1 => One,
                    _ => Ident(choice - 2),
                }
            }
            2 => {
                // UnaOp (op1_len)
                let op = self.rng.choose(self.op1_choices);
                let expr = self.gen_expr(1, idents, foldable, false);
                Op1(op, ~expr)
            }
            3 => {
                // Choices:
                // 1. UnaOp (op1_len)
                // 2. BinOp (op2_len)
                match self.rng.gen_uint_range(0, self.op1_len + self.op2_len) {
                    n if n < self.op1_len => { // UnaOp
                        let expr = self.gen_expr(2, idents, foldable, false);
                        Op1(self.op1_choices[n], ~expr)
                    }
                    n => {
                        let left = self.gen_expr(1, idents, foldable, false);
                        let right = self.gen_expr(1, idents, foldable, false);
                        Op2(self.op2_choices[n - self.op1_len], ~left, ~right)
                    }
                }
            }
            4 => {
                // Choices:
                // 1. UnaOp (op1_len)
                // 2. BinOp (op2_len) * (2 = spaces - 1)
                // 3. If0 (1?)
                let if_len = if self.operators.if0 { 1 } else { 0 };
                match self.rng.gen_uint_range(0, self.op1_len + self.op2_len * 2 + if_len) {
                    n if n < self.op1_len => {
                        let expr = self.gen_expr(3, idents, foldable, false);
                        Op1(self.op1_choices[n], ~expr)
                    }
                    n if n < self.op2_len * 2 => {
                        let left_bigger = self.rng.gen::<bool>();
                        let (left_size, right_size) = if left_bigger {
                            (2, 1)
                        } else {
                            (1, 2)
                        };
                        let left = self.gen_expr(left_size, idents, foldable, false);
                        let right = self.gen_expr(right_size, idents, foldable, false);
                        let op = self.rng.gen::<BinOp>();
                        Op2(op, ~left, ~right)
                    }
                    _ => {
                        let test = self.gen_expr(1, idents, foldable, false);
                        let then = self.gen_expr(1, idents, foldable, false);
                        let other = self.gen_expr(1, idents, foldable, false);
                        If0(~test, ~then, ~other)
                    }
                }
            }
            _ => {
                // Choices:
                // 1. UnaOp (op1_len)
                // 2. BinOp (op2_len) * (spaces - 1)
                // 3. If0 ((spaces - 1) choose 2 = 1/2 * (n - 1) * (n - 2))
                // 4. Fold (1) [only if foldable && !root]
                let spaces = size - 1;
                let spaces_choose_2 = spaces * (spaces - 1) / 2;
                let mut choices = self.op1_len + (self.op2_len * (spaces - 1));
                if self.operators.if0 {
                    choices += spaces_choose_2;
                }
                if foldable && !root {
                    choices += spaces_choose_2;
                }

                let op2_end = self.op1_len + (self.op2_len * (spaces - 1));
                let if_end = if self.operators.if0 {
                    op2_end + spaces_choose_2
                } else {
                    op2_end // skipped
                };

                match self.rng.gen_uint_range(0, choices) {
                    n if n < self.op1_len => {
                        let expr = self.gen_expr(size - 1, idents, foldable, false);
                        Op1(self.op1_choices[n], ~expr)
                    }
                    n if n < op2_end => {
                        let left_size = self.rng.gen_uint_range(1, size - 2);
                        let right_size = size - 1 - left_size;
                        let left = self.gen_expr(left_size, idents, foldable, false);
                        let right = self.gen_expr(right_size, idents, foldable, false);
                        let op = self.rng.gen::<BinOp>();
                        Op2(op, ~left, ~right)
                    }
                    n if n < if_end => {
                        let test_size = self.rng.gen_uint_range(1, size - 3);
                        let rest = size - 1 - test_size;
                        let then_size = self.rng.gen_uint_range(1, rest - 1);
                        let other_size = size - 1 - test_size - then_size;
                        let test = self.gen_expr(test_size, idents, foldable, false);
                        let then = self.gen_expr(then_size, idents, foldable, false);
                        let other = self.gen_expr(other_size, idents, foldable, false);
                        If0(~test, ~then, ~other)
                    }
                    _ => {
                        let foldee_size = self.rng.gen_uint_range(1, size - 3);
                        let rest = size - 1 - foldee_size;
                        let init_size = self.rng.gen_uint_range(1, rest - 1);
                        let body_size = size - 1 - foldee_size - init_size;
                        let foldee = self.gen_expr(foldee_size, idents, foldable, false);
                        let init = self.gen_expr(init_size, idents, foldable, false);
                        let body = self.gen_expr(body_size, idents + 2, foldable, false);
                        Fold {
                            foldee: ~foldee,
                            init: ~init,
                            next_id: 1,
                            accum_id: 2,
                            body: ~body,
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use extra::test::BenchHarness;
    use super::*;
    use webapi::*;
    use program::*;

    #[bench]
    fn bench_gen_prog(bh: &mut BenchHarness) {
        let mut opset = OperatorSet::new();
        opset.add(~[~"shl1", ~"not", ~"and", ~"xor", ~"if0"]);
        let problem = Problem {
            id: ~"asdf",
            size: 30,
            operators: opset,
        };
        let mut gen = RandomGen::new(problem, ~[]);
        do bh.iter {
            gen.next();
        }
    }
}

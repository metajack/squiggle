use webapi::*;
use program::*;

use std::cell::Cell;
use std::comm;
use std::comm::{Port, Chan};
use std::rand::{Rng, RngUtil, XorShiftRng, task_rng};
use std::task;
use extra::arc;

static PARALLELISM: uint = 1;
static CHECK_EVERY: uint = 1024;

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
                    let (inner_port, inner_chan) = stream();
                    let inner_chan = comm::SharedChan::new(inner_chan);
                    let stop_arc = arc::RWArc::new(false);
                    let problem_size = problem.size as uint;

                    for task_num in range(0, PARALLELISM) {
                        let task_chan = inner_chan.clone();
                        let task_stop_arc = stop_arc.clone();
                        let task_gen = Cell::new(gen.clone());
                        let task_constraints = constraints.clone();

                        do spawn {
                            let mut task_gen = task_gen.take();

                            let mut i = 0;
                            'newprog: loop {
                                i += 1;
                                if i % CHECK_EVERY == 0 {
                                    if task_stop_arc.read(|&stop| stop) {
                                        printfln!("task %u: someone else found it", task_num);
                                        break
                                    }

                                    // required for any parallelism at all.
                                    task::yield();
                                }

                                let prog = task_gen.gen_program(problem_size);

                                if task_constraints.iter().any(|&(x,y)| prog.eval(x) != y) {
                                    if i % 1000000 == 0 {
                                        printfln!("gen stats: task %u: searched for %u iters",
                                                  task_num, i);
                                    }
                                    loop 'newprog;
                                }
                                if i > 1 {
                                    printfln!("gen stats: task %u: candidate took %u iters",
                                              task_num, i);
                                }

                                task_chan.send(~prog);
                                break;
                            }
                        }
                    }

                    let prog = inner_port.recv();
                    stop_arc.write(|stop| *stop = true);

                    chan.send(prog);
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

impl Clone for RandomGenState {
    fn clone(&self) -> RandomGenState {
        RandomGenState {
            rng: seeded_rng(), // need a new rng
            operators: self.operators.clone(),
            op1_len: self.op1_len,
            op1_choices: self.op1_choices.clone(),
            op2_len: self.op2_len,
            op2_choices: self.op2_choices.clone(),
        }
    }
}

fn seeded_rng() -> XorShiftRng {
    let mut seed_rng = task_rng();
    XorShiftRng::new_seeded(seed_rng.gen::<u32>(),
                            seed_rng.gen::<u32>(),
                            seed_rng.gen::<u32>(),
                            seed_rng.gen::<u32>())
}


impl RandomGenState {
    fn new(problem: Problem) -> RandomGenState {
        let rng = seeded_rng();

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

        if self.operators.tfold {
            // remove the sizes of fold, x and 0
            let body = ~self.gen_expr(size - 2 - 1 - 1, 2, false);

            // use 2 here, because it won't be referred to in the body
            // ever. i.e. it's shadowed.
            Program::new(2, ~Fold {
                    foldee: ~Ident(2),
                    init: ~Zero,
                    accum_id: 0,
                    next_id: 1,
                    body: body
                })
        } else {
            Program::new(0, ~self.gen_expr(size, 1, self.operators.fold))
        }
    }

    fn gen_expr(&mut self, size: uint, idents: uint, foldable: bool) -> Expr {
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
                assert!(self.op1_len > 0);

                let op = self.rng.choose(self.op1_choices);
                let expr = self.gen_expr(1, idents, foldable);
                Op1(op, ~expr)
            }
            3 => {
                // Choices:
                // 1. UnaOp (op1_len)
                // 2. BinOp (op2_len)
                match self.rng.gen_uint_range(0, self.op1_len + self.op2_len) {
                    n if n < self.op1_len => { // UnaOp
                        assert!(self.op1_len > 0);

                        let expr = self.gen_expr(2, idents, foldable);
                        Op1(self.op1_choices[n], ~expr)
                    }
                    n => {
                        assert!(self.op2_len > 0);

                        let left = self.gen_expr(1, idents, foldable);
                        let right = self.gen_expr(1, idents, foldable);
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

                // If there are no unaops, then we must be able to place an if
                assert!(self.op1_len > 0 || self.operators.if0);

                match self.rng.gen_uint_range(0, self.op1_len + self.op2_len * 2 + if_len) {
                    n if n < self.op1_len => {
                        assert!(self.op1_len > 0);

                        let expr = self.gen_expr(3, idents, foldable);
                        Op1(self.op1_choices[n], ~expr)
                    }
                    n if n < self.op1_len + self.op2_len * 2 => {
                        assert!(self.op2_len > 0);

                        let left_bigger = self.rng.gen::<bool>();
                        let (left_size, right_size) = if left_bigger {
                            (2, 1)
                        } else {
                            (1, 2)
                        };
                        let left = self.gen_expr(left_size, idents, foldable);
                        let right = self.gen_expr(right_size, idents, foldable);
                        let op = self.rng.gen::<BinOp>();
                        Op2(op, ~left, ~right)
                    }
                    _ => {
                        assert!(self.operators.if0);

                        let test = self.gen_expr(1, idents, foldable);
                        let then = self.gen_expr(1, idents, foldable);
                        let other = self.gen_expr(1, idents, foldable);
                        If0(~test, ~then, ~other)
                    }
                }
            }
            _ => {
                // Choices:
                // 1. UnaOp (op1_len)
                // 2. BinOp (op2_len) * (spaces - 1)
                // 3. If0 ((spaces - 1) choose 2 = 1/2 * (n - 1) * (n - 2))
                // 4. Fold (1) [only if foldable]
                let spaces = size - 1;
                let spaces_choose_2 = spaces * (spaces - 1) / 2;
                let mut choices = self.op1_len + (self.op2_len * (spaces - 1));
                if self.operators.if0 {
                    choices += spaces_choose_2;
                }
                if foldable {
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
                        assert!(self.op1_len > 0);

                        let expr = self.gen_expr(size - 1, idents, foldable);
                        Op1(self.op1_choices[n], ~expr)
                    }
                    n if n < op2_end => {
                        assert!(self.op2_len > 0);

                        let size = size - 1; // account for op
                        let left_size = self.gen_size(size - 1);
                        let right_size = size - left_size;
                        let left = self.gen_expr(left_size, idents, foldable);
                        let right = self.gen_expr(right_size, idents, foldable);
                        let op = self.rng.gen::<BinOp>();
                        Op2(op, ~left, ~right)
                    }
                    n if n < if_end => {
                        assert!(self.operators.if0);

                        let test_size = self.gen_size(size - 3);
                        let rest = size - 1 - test_size;
                        let then_size = self.gen_size(rest - 1);
                        let other_size = size - 1 - test_size - then_size;
                        let test = self.gen_expr(test_size, idents, foldable);
                        let then = self.gen_expr(then_size, idents, foldable);
                        let other = self.gen_expr(other_size, idents, foldable);
                        If0(~test, ~then, ~other)
                    }
                    _ => {
                        assert!(foldable);

                        let size = size - 2; // account for |fold|.

                        // need to leave at least 2 spaces for the
                        // init and body. (this generates in `[1, size
                        // - 1)`, i.e. the largest is size - 2)
                        let foldee_size = self.gen_size(size - 2);
                        let rest = size - foldee_size;
                        let init_size = self.gen_size(rest - 1);
                        let body_size = rest - init_size;

                        let foldee = self.gen_expr(foldee_size, idents, false);
                        let init = self.gen_expr(init_size, idents, false);
                        let body = self.gen_expr(body_size, idents + 2, false);

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

    // when we genrrate ranges, it might be the case that we can't place a
    // unaop, so size=2 must be prevented this warps a size=2 choice randomly
    // up or down (or only down if size < 3.
    //
    // additionally, we prevent size=4 if there is no if0, since that forces
    // binops, cause one of the arguments to be of size=2
    fn gen_size(&mut self, space: uint) -> uint {
        assert!(space >= 1);

        let choice = self.rng.gen_uint_range(1, space + 1);
        match (choice, self.op1_len > 0) {
            (2, true) => choice, // don't need to do anything since we have unaops
            (2, false) if space <= 2 => 1, // not enough space for 3 so go down
            (2, false) => { // pick from (1,3) at random
                if self.rng.gen() {
                    1
                } else {
                    3
                }
            }
            (4, false) if self.operators.if0 => choice, // if we have if we are safe
            (4, false) => { // can't use 4 so go up or down
                if space >= 5 {
                    if self.rng.gen() { 3 } else { 5 }
                } else {
                    3
                }
            }
            _ => choice
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

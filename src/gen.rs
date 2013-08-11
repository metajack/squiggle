use webapi::*;
use program::*;

use std::cell::Cell;
use std::comm;
use std::comm::{Port, Chan};
use std::from_str::FromStr;
use std::os;
use std::rand::{Rng, RngUtil, XorShiftRng, task_rng};
use std::task;
use extra::arc;
use extra::time;

static DEFAULT_PARALLELISM: uint = 0;
static CHECK_EVERY: uint = 10_000;

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

                    let start_ns = time::precise_time_ns();

                    let parallelism: uint = FromStr::from_str(
                        os::getenv("PAR").unwrap_or_default(~"1"))
                        .unwrap_or_default(DEFAULT_PARALLELISM);

                    for task_num in range(0, parallelism) {
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
                                //println(prog.to_str());

                                if task_constraints.iter().any(|&(x,y)| prog.eval(x) != y) {
                                    if i % 1000000 == 0 {
                                        let elapsed = time::precise_time_ns() - start_ns;
                                        printfln!("gen stats: task %u: searched for %uMiter (%uns/iter)",
                                                  task_num, i / 1_000_000, (elapsed / (i as u64)) as uint);
                                    }
                                    loop 'newprog;
                                }
                                if i > 1 {
                                    let elapsed = time::precise_time_ns() - start_ns;
                                    printfln!("gen stats: task %u: candidate took %uMiter %ums",
                                              task_num, i / 1000000, (elapsed / 1000000) as uint);
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

    fn gen_program(&mut self, size: uint) -> Program {
        //println("gen_program");
        if self.operators.tfold {
            self.gen_tfold(size)
        } else if self.operators.bonus {
            self.gen_bonus(size)
        } else {
            Program::new(0, ~self.gen_expr(size - 1, 1, self.operators.fold))
        }
    }

    fn gen_tfold(&mut self, size: uint) -> Program {
        // remove the sizes of the program, fold, x and 0
        let body = ~self.gen_expr(size - 1 - 2 - 1 - 1, 2, false);

        // use 2 here, because it won't be referred to in the body
        // ever. i.e. it's shadowed.
        Program::new(2, ~Fold {
                foldee: ~Ident(2),
                init: ~Zero,
                next_id: 0,
                accum_id: 1,
                body: body
            })
    }

    fn gen_bonus(&mut self, size: uint) -> Program {
        //let

        //let expr = ~If0(~And(,~One), )

        fail!()
    }

    fn gen_expr(&mut self, size: uint, idents: uint, foldable: bool) -> Expr {
        //printfln!("gen_expr(%u)", size);
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

                        //println("genning left");
                        let left = self.gen_expr(1, idents, foldable);
                        //println("genning right");
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

                // can only gen binops if we have unaops, otherwise we'd have a 2/1 slot split and 2 is bad
                let gen_binop = self.op1_len > 0;

                // If there are no unaops, then we must be able to place an if or fold
                assert!(self.op1_len > 0 || self.operators.if0 || foldable);

                let mut choices = self.op1_len;
                let op1_end = self.op1_len;
                if gen_binop {
                    choices += self.op2_len;
                }
                let op2_end = choices;

                if self.operators.if0 {
                    choices += 1;
                }
                let if_end = choices;

                match self.rng.gen_uint_range(0, choices) {
                    n if n < op1_end => {
                        assert!(self.op1_len > 0);

                        let expr = self.gen_expr(3, idents, foldable);
                        Op1(self.op1_choices[n], ~expr)
                    }
                    n if n < op2_end => {
                        //println("genning binop");
                        assert!(self.op2_len > 0);

                        let left_bigger = self.rng.gen::<bool>();
                        let (left_size, right_size) = if left_bigger {
                            (2, 1)
                        } else {
                            (1, 2)
                        };
                        //println("genning left");
                        let left = self.gen_expr(left_size, idents, foldable);
                        //println("genning right");
                        let right = self.gen_expr(right_size, idents, foldable);
                        let op = self.rng.choose(self.op2_choices);
                        Op2(op, ~left, ~right)
                    }
                    n if n < if_end => {
                        //println("genning if0");
                        assert!(self.operators.if0);

                        //println("genning test");
                        let test = self.gen_expr(1, idents, foldable);
                        //println("genning then");
                        let then = self.gen_expr(1, idents, foldable);
                        //println("genning other");
                        let other = self.gen_expr(1, idents, foldable);
                        If0(~test, ~then, ~other)
                    }
                    _ => fail!("bad choice"),
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

                let mut choices = self.op1_len;

                let have_unaops = self.op1_len > 0;
                let have_if0 = self.operators.if0;

                // we can't generate even sized binops without unaops or if
                let gen_binop = have_unaops || size.is_odd() || have_if0;
                if gen_binop {
                    choices += (self.op2_len * (spaces - 1));
                }

                // we can't generate size=5,6 if0s without unaops as it will
                // force a size=2. size=4 is handled by previous match arm.
                let gen_if = have_if0 && (have_unaops || size > 6);
                //printfln!("can gen if? %b", gen_if);

                // we can't generate size=6,7 folds without unaops as it will
                // force a size=2. we also can only gen size=8,10 if we have if0
                let gen_fold = foldable && (have_unaops ||
                                            (have_if0 && (size == 8 || size == 10)) ||
                                            (size != 5 && size != 6 && size != 8 && size != 10));

                if gen_if {
                    choices += spaces_choose_2;
                }
                if gen_fold {
                    choices += spaces_choose_2;
                }

                let op2_end = if gen_binop {
                    self.op1_len + (self.op2_len * (spaces - 1))
                } else {
                    self.op1_len
                };

                let if_end = if gen_if {
                    op2_end + spaces_choose_2
                } else {
                    op2_end // skipped
                };

                //printfln!("size=%u and choices=%u", size, choices);
                match self.rng.gen_uint_range(0, choices) {
                    n if n < self.op1_len => {
                        //println("genning unaop");
                        assert!(self.op1_len > 0);

                        let expr = self.gen_expr(size - 1, idents, foldable);
                        Op1(self.op1_choices[n], ~expr)
                    }
                    n if n < op2_end => {
                        //println("genning binop");
                        assert!(self.op2_len > 0);

                        let size = size - 1; // account for op
                        let mut left_size;
                        let mut right_size;
                        loop {
                            left_size = self.gen_size(size - 1, foldable);
                            right_size = size - left_size;

                            if self.check_size(right_size, foldable) { break; }

                            //printfln!("looping with left=%u and right=%u", left_size, right_size);
                        }
                        //printfln!("picked left=%u and right=%u", left_size, right_size);

                        //println("genning left");
                        let left = self.gen_expr(left_size, idents, foldable);
                        //println("genning right");
                        let right = self.gen_expr(right_size, idents, foldable);
                        let op = self.rng.choose(self.op2_choices);
                        Op2(op, ~left, ~right)
                    }
                    n if n < if_end => {
                        //printfln!("genning if size %u", size);
                        assert!(self.operators.if0);
                        assert!(self.op1_len > 0 || size != 5);

                        let size = size - 1; // acount for |if|

                        let mut test_size;
                        let mut then_size;
                        let mut other_size;
                        loop {
                            test_size = self.gen_size(size - 2, foldable);
                            let rest = size - test_size;
                            then_size = self.gen_size(rest - 1, foldable);
                            other_size = rest - then_size;

                            if self.check_size(other_size, foldable) { break; }

                            //printfln!("looping with test=%u, then=%u, other=%u", test_size, then_size, other_size);
                        }
                        //printfln!("picked test=%u, then=%u, other=%u", test_size, then_size, other_size);

                        //println("genning test");
                        let test = self.gen_expr(test_size, idents, foldable);
                        //println("genning then");
                        let then = self.gen_expr(then_size, idents, foldable);
                        //println("genning other");
                        let other = self.gen_expr(other_size, idents, foldable);
                        If0(~test, ~then, ~other)
                    }
                    _ => {
                        //println("genning fold");
                        assert!(foldable);

                        let size = size - 2; // account for |fold|.

                        let mut foldee_size;
                        let mut init_size;
                        let mut body_size;
                        //printfln!("size = %u", size);
                        loop {
                            //printfln!("size-2=%u", size - 2);
                            foldee_size = self.gen_size(size - 2, false);
                            let rest = size - foldee_size;
                            //printfln!("picked %u rest = %u", foldee_size, rest);
                            init_size = self.gen_size(rest - 1, false);
                            //printfln!("picked %u rest = %u", init_size, rest - init_size);
                            body_size = rest - init_size;

                            if self.check_size(body_size, false) { break; }

                            //printfln!("looping with foldee=%u, init=%u, body=%u", foldee_size, init_size, body_size);
                        }
                        //printfln!("picked foldee=%u, init=%u, body=%u", foldee_size, init_size, body_size);

                        //println("genning foldee");
                        let foldee = self.gen_expr(foldee_size, idents, false);
                        //println("genning init");
                        let init = self.gen_expr(init_size, idents, false);
                        //println("genning body");
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

    // Generate a size for a slot
    //
    // If we can use UnaOps, this is trivial, as all sizes are valid.
    //
    // Without UnaOps, we cannot generate any size=2.
    //
    // Without if0 or fold, we can't generate any even sizes.
    //
    // If we have if0 or fold, we can't generate a size=6, as this would force
    // a 2 when split three ways.
    fn gen_size(&mut self, space: uint, foldable: bool) -> uint {
        //printfln!("gen_size(%u)", space);
        assert!(space >= 1);

        let have_unaops = self.op1_len > 0;
        let have_fold_or_if0 = foldable || self.operators.if0;

        let choice = self.rng.gen_uint_range(1, space + 1);

        // If we have unaops we're done.
        if have_unaops { return choice; }

        // If we don't have if0 or fold, force it to be odd.
        if !have_fold_or_if0 {
            if choice.is_even() {
                if space > choice && self.rng.gen() {
                    return choice + 1;
                } else {
                    return choice - 1;
                }
            } else {
                return choice;
            }
        }

        // We have fold/if0, so prevent 6 and 2. 6 causes a slot of 2, and 2
        // is not allowed because of missing unaops
        if choice == 6 || choice == 2 {
            if space > choice && self.rng.gen() {
                return choice + 1;
            } else {
                return choice - 1;
            }
        }

        // If we only have fold prevent 4 which is too small for fold.
        if foldable && choice == 4 {
            if space > choice && self.rng.gen() {
                return choice + 1;
            } else {
                return choice - 1;
            }
        }

        // everythign is ok
        choice
    }

    // Check a size for validity
    //
    // Check the rules above for a size. This is needed for leftover slots.
    fn check_size(&mut self, choice: uint, foldable: bool) -> bool {
        //printfln!("check_size(%u)", choice);
        let have_unaops = self.op1_len > 0;
        let have_fold_or_if0 = foldable || self.operators.if0;

        if have_unaops { return true; }
        if !have_fold_or_if0 && choice.is_even() { return false; }

        // 2 is too small for if, so including that
        choice != 6 && choice != 2
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

    #[test]
    fn SqTTVYnARE6FeVMn9VNsUOTa() {
        let mut opset = OperatorSet::new();
        opset.add(~[~"if0", ~"and", ~"or", ~"plus"]);
        let problem = Problem {
            id: ~"test-SqTTVYnARE6FeVMn9VNsUOTa",
            size: 11,
            operators: opset,
        };
        let mut gen = RandomGen::new(problem, ~[]);
        for _ in range(0, 10) {
            gen.next();
        }
    }

    #[test]
    fn no_unaops_noif_fold() {
        let mut opset = OperatorSet::new();
        opset.add(~[~"and", ~"or", ~"plus", ~"xor", ~"fold"]);
        let problem = Problem {
            id: ~"no_unaops_noif_fold",
            size: 12,
            operators: opset,
        };
        let mut gen = RandomGen::new(problem, ~[]);
        for _ in range(0, 10) {
            gen.next();
        }
    }
}

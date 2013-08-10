use std::str;
pub type Id = uint;

fn id_to_str(mut num: Id) -> ~str {
    let mut s = str::with_capacity(5);
    loop {
        let (div, rem) = num.div_rem(&26);
        s.push_char((rem + 97) as char);
        if div == 0 {
            break;
        }
        num = div;
    }
    s
}

pub struct OperatorSet {
    op1: [bool, ..5],
    op2: [bool, ..4],
    if0: bool,
    fold: bool,
    tfold: bool,
}

impl OperatorSet {
    pub fn new() -> OperatorSet {
        OperatorSet {
            op1: [false, false, false, false, false],
            op2: [false, false, false, false],
            if0: false,
            fold: false,
            tfold: false,
        }
    }

    pub fn add(&mut self, ops: ~[~str]) {
        for op in ops.iter() {
            match *op {
                ~"not" => self.op1[Not as uint] = true,
                ~"shl1" => self.op1[Shl1 as uint] = true,
                ~"shr1" => self.op1[Shr1 as uint] = true,
                ~"shr4" => self.op1[Shr4 as uint] = true,
                ~"shr16" => self.op1[Shr16 as uint] = true,
                ~"and" => self.op2[And as uint] = true,
                ~"or" => self.op2[Or as uint] = true,
                ~"xor" => self.op2[Xor as uint] = true,
                ~"plus" => self.op2[Plus as uint] = true,
                ~"if0" => self.if0 = true,
                ~"fold" => self.fold = true,
                ~"tfold" => self.tfold = true,
                _ => fail!("bad operation"),
            }
        }
    }

    fn add_from_expr(&mut self, e: &Expr) {
        match *e {
            Op1(op, ~ref sub_e) => {
                self.op1[op as uint] = true;
                println("op1");
                self.add_from_expr(sub_e)
            }
            Op2(op, ~ref lhs, ~ref rhs) => {
                self.op2[op as uint] = true;
                println("op2");
                self.add_from_expr(lhs);
                self.add_from_expr(rhs);
            }
            If0(~ref c, ~ref t, ~ref e) => {
                self.if0 = true;
                println("if0");
                self.add_from_expr(c);
                self.add_from_expr(t);
                self.add_from_expr(e);
            }
            Fold {foldee: ~ref foldee, init: ~ref init, body: ~ref body, _ } => {
                self.fold = true;
                println("fold");
                self.add_from_expr(foldee);
                self.add_from_expr(init);
                self.add_from_expr(body);
            }
            Ident(_) | One | Zero => {} // no operations
        }
    }

    pub fn add_from_program(&mut self, p: &Program) {
        match *p.expr {
            // need to handle tfold specially; note that this isn't
            // quite right (since it doesn't check for shadowing)
            Fold {foldee: ~ref foldee, init: ~ref init, body: ~ref body, _ } => {
                match (foldee, init) {
                    (&Ident(_), &Zero) => {
                        self.tfold = true;
                        self.add_from_expr(body);
                        return;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        self.add_from_expr(p.expr)
    }
}

impl ToStr for OperatorSet {
    pub fn to_str(&self) -> ~str {
        let mut ops = ~[];
        if self.tfold { ops.push(~"tfold"); }
        if self.fold { ops.push(~"fold"); }
        if self.if0 { ops.push(~"if0"); }
        if self.op1[0] { ops.push(~"not"); }
        if self.op1[1] { ops.push(~"shl1"); }
        if self.op1[2] { ops.push(~"shr1"); }
        if self.op1[3] { ops.push(~"shr4"); }
        if self.op1[4] { ops.push(~"shr16"); }
        if self.op2[0] { ops.push(~"and"); }
        if self.op2[1] { ops.push(~"or"); }
        if self.op2[2] { ops.push(~"xor"); }
        if self.op2[3] { ops.push(~"plus"); }
        ops.connect(",")
    }
}

impl Clone for OperatorSet {
    pub fn clone(&self) -> OperatorSet {
        let mut ops = OperatorSet::new();
        for i in range(0, 5) {
            ops.op1[i] = self.op1[i];
            if i != 4 {
                ops.op2[i] = self.op2[i];
            }
        }
        ops.if0 = self.if0;
        ops.fold = self.fold;
        ops.tfold = self.tfold;
        ops
    }
}

impl Eq for OperatorSet {
    pub fn eq(&self, other: &OperatorSet) -> bool {
        for i in range(0, 5) {
            if other.op1[i] != self.op1[i] { return false; }
            if i != 4 {
                if other.op2[i] != self.op2[i] { return false; }
            }
        }
        if other.if0 != self.if0 { return false; }
        if other.fold != self.fold { return false; }
        if other.tfold != self.tfold { return false; }
        true
    }
}

#[deriving(Eq, Clone)]
pub struct Program {
    id: Id,
    expr: ~Expr,
}

pub static OP1_CHOICE: [UnaOp, ..5] = [Not, Shl1, Shr1, Shr4, Shr16];

#[deriving(Rand,Eq, Clone)]
pub enum UnaOp {
    Not = 0,
    Shl1 = 1,
    Shr1 = 2,
    Shr4 = 3,
    Shr16 = 4,
}

impl UnaOp {
    fn in_ops(&self, ops: &OperatorSet) -> bool {
        ops.op1[*self as uint]
    }
}

pub static OP2_CHOICE: [BinOp, ..4] = [And, Or, Xor, Plus];

#[deriving(Rand,Eq, Clone)]
pub enum BinOp {
    And = 0,
    Or = 1,
    Xor = 2,
    Plus = 3,
}

impl BinOp {
    fn in_ops(&self, ops: &OperatorSet) -> bool {
        ops.op2[*self as uint]
    }
}

#[deriving(Eq, Clone)]
pub enum Expr {
    Zero,
    One,
    Ident(Id),
    If0(~Expr, ~Expr, ~Expr),
    Op1(UnaOp, ~Expr),
    Op2(BinOp, ~Expr, ~Expr),
    Fold {
        foldee: ~Expr,
        init: ~Expr,
        next_id: Id,
        accum_id: Id,
        body: ~Expr
    }
}

impl Program {
    pub fn new(id: Id, expr: ~Expr) -> Program {
        Program {
            id: id,
            expr: expr,
        }
    }

    pub fn len(&self) -> u8 {
        1 + self.expr.len()
    }

    pub fn operators(&self) -> OperatorSet {
        let mut operators = OperatorSet::new();
        operators.add_from_program(self);
        operators
    }
}

impl Expr {
    pub fn len(&self) -> u8 {
        match *self {
            Zero => 1,
            One => 1,
            Ident(*) => 1,
            If0(ref test, ref then, ref other) => {
                1 + test.len() + then.len() + other.len()
            }
            Op1(_, ref expr) => 1 + expr.len(),
            Op2(_, ref left, ref right) => 1 + left.len() + right.len(),
            Fold {foldee: ref foldee, init: ref init, body: ref body, _ } => {
                2 + foldee.len() + init.len() + body.len()
            }
        }
    }
}

impl FromStr for Program {
    pub fn from_str(_s: &str) -> Option<Program> {
        None
    }
}

impl ToStr for Program {
    pub fn to_str(&self) -> ~str {
        fmt!("(lambda (%s) %s)", id_to_str(self.id), self.expr.to_str())
    }
}

impl ToStr for UnaOp {
    pub fn to_str(&self) -> ~str {
        match *self {
            Not => ~"not",
            Shl1 => ~"shl1",
            Shr1 => ~"shr1",
            Shr4 => ~"shr4",
            Shr16 => ~"shr16",
        }
    }
}

impl ToStr for BinOp {
    pub fn to_str(&self) -> ~str {
        match *self {
            And => ~"and",
            Or => ~"or",
            Xor => ~"xor",
            Plus => ~"plus",
        }
    }
}

impl ToStr for Expr {
    pub fn to_str(&self) -> ~str {
        match *self {
            Zero => ~"0",
            One => ~"1",
            Ident(id) => id_to_str(id),
            If0(ref test, ref then, ref other) => {
                fmt!("(if0 %s %s %s)", test.to_str(), then.to_str(), other.to_str())
            }
            Op1(op, ref expr) => fmt!("(%s %s)", op.to_str(), expr.to_str()),
            Op2(op, ref left, ref right) => {
                fmt!("(%s %s %s)", op.to_str(), left.to_str(), right.to_str())
            }
            Fold {
                foldee: ref foldee, init: ref init,
                accum_id, next_id,
                body: ref body
            } => {
                fmt!("(fold %s %s (lambda (%s %s) %s))",
                     foldee.to_str(), init.to_str(),
                     id_to_str(next_id), id_to_str(accum_id),
                     body.to_str())
            }
        }
    }
}

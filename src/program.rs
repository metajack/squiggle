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
                ~"not" => self.op1[OP_NOT] = true,
                ~"shl1" => self.op1[OP_SHL1] = true,
                ~"shr1" => self.op1[OP_SHR1] = true,
                ~"shr4" => self.op1[OP_SHR4] = true,
                ~"shr16" => self.op1[OP_SHR16] = true,
                ~"and" => self.op2[OP_AND] = true,
                ~"or" => self.op2[OP_OR] = true,
                ~"xor" => self.op2[OP_XOR] = true,
                ~"plus" => self.op2[OP_PLUS] = true,
                ~"if0" => self.if0 = true,
                ~"fold" => self.fold = true,
                ~"tfold" => self.tfold = true,
                _ => fail!("bad operation"),
            }
        }
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

#[deriving(Eq)]
pub struct Program {
    id: Id,
    expr: ~Expr,
}

pub static OP_NOT: uint = 0;
pub static OP_SHL1: uint = 1;
pub static OP_SHR1: uint = 2;
pub static OP_SHR4: uint = 3;
pub static OP_SHR16: uint = 4;

#[deriving(Rand,Eq,IterBytes)]
pub enum UnaOp {
    Not,
    Shl1,
    Shr1,
    Shr4,
    Shr16,
}

impl UnaOp {
    fn in_ops(&self, ops: &OperatorSet) -> bool {
        match *self {
            Not => ops.op1[OP_NOT],
            Shl1 => ops.op1[OP_SHL1],
            Shr1 => ops.op1[OP_SHR1],
            Shr4 => ops.op1[OP_SHR4],
            Shr16 => ops.op1[OP_SHR16],
        }
    }
}

pub static OP_AND: uint = 0;
pub static OP_OR: uint = 1;
pub static OP_XOR: uint = 2;
pub static OP_PLUS: uint = 3;

#[deriving(Rand,Eq,IterBytes)]
pub enum BinOp {
    And,
    Or,
    Xor,
    Plus,
}

impl BinOp {
    fn in_ops(&self, ops: &OperatorSet) -> bool {
        match *self {
            And => ops.op2[OP_AND],
            Or => ops.op2[OP_OR],
            Xor => ops.op2[OP_XOR],
            Plus => ops.op2[OP_PLUS],
        }
    }
}

#[deriving(Eq)]
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
                let mut e = ~"(if0 ";
                e.push_str(test.to_str());
                e.push_str(" ");
                e.push_str(then.to_str());
                e.push_str(" ");
                e.push_str(other.to_str());
                e
            }
            Op1(op, ref expr) => {
                let mut e = ~"(";
                e.push_str(op.to_str());
                e.push_str(" ");
                e.push_str(expr.to_str());
                e.push_str(")");
                e
            }
            Op2(op, ref left, ref right) => {
                let mut e = ~"(";
                e.push_str(op.to_str());
                e.push_str(" ");
                e.push_str(left.to_str());
                e.push_str(" ");
                e.push_str(right.to_str());
                e.push_str(")");
                e
            }
            Fold {
                foldee: ref foldee, init: ref init,
                accum_id, next_id,
                body: ref body
            } => {
                fmt!("(fold %s %s (lambda (%s %s) %s))",
                     foldee.to_str(), init.to_str(),
                     id_to_str(accum_id), id_to_str(next_id),
                     body.to_str())
            }
        }
    }
}

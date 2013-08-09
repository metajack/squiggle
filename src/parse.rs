use program::Expr;

pub type ParseResult = Result<~Expr, ~str>;

pub trait Parse {
    pub fn parse(&self) -> ParseResult;
}

impl<'self> Parse for &'self str {
    pub fn parse(&self) -> ParseResult {
        Parser {
            src_pos: 0,
            src: *self,
        }.parse()
    }
}

pub struct Parser<'self> {
    priv src_pos: uint,
    priv src: &'self str,
}

impl<'self> Parse for Parser<'self> {
    pub fn parse(&self) -> ParseResult {
        self.consume_lambda()
    }
}

impl<'self> Parser<'self> {
    pub fn consume_ws(&self) -> ParseResult {
        Err(~"Not yet implemented.")
    }
    pub fn consume_str(&self, _expected: &str) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn consume_lambda(&self) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn comsume_expression(&self) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn consume_op1(&self) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn consume_op2(&self) -> ParseResult {
        Err(~"Not yet implemented.")
    }

    pub fn consume_id(&self) -> ParseResult {
        Err(~"Not yet implemented.")
    }
}
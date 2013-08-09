use program::Expr;

pub type ParseResult = Result<~Expr, ~str>;

pub fn parse(input: &str) -> ParseResult {
    consume_lambda(input)
}

pub fn consume_lambda(_input: &str) -> ParseResult {
    Err(~"Not yet implemented.")
}

pub fn comsume_expression(_input: &str) -> ParseResult {
    Err(~"Not yet implemented.")
}

pub fn consume_op1(_input: &str) -> ParseResult {
    Err(~"Not yet implemented.")
}

pub fn consume_op2(_input: &str) -> ParseResult {
    Err(~"Not yet implemented.")
}

pub fn consume_id(_input: &str) -> ParseResult {
    Err(~"Not yet implemented.")
}

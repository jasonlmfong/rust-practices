use std::{error, fmt, fmt::Display, iter::Peekable, str::Chars};

#[derive(PartialEq, Debug)]
pub enum ExpressionError {
    Parsing(String),
}

// This is required so that `ExpressionError` can implement `error::Error`.
impl fmt::Display for ExpressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ExpressionError::Parsing(ref description) = *self;
        f.write_str(description)
    }
}

impl error::Error for ExpressionError {}

#[derive(PartialEq, Debug)]
enum Associative {
    Left,
    Right,
}

// tokens/symbols in an expression
#[derive(Debug, Clone, Copy)]
enum Token {
    Number(i32),
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    LeftParenthesis,
    RightParenthesis,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt_str = match self {
            Token::Number(n) => n.to_string(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Multiply => "*".to_string(),
            Token::Divide => "/".to_string(),
            Token::Power => "^".to_string(),
            Token::LeftParenthesis => "(".to_string(),
            Token::RightParenthesis => ")".to_string(),
        };

        write!(f, "{}", fmt_str)
    }
}

impl Token {
    fn is_operator(&self) -> bool {
        match self {
            Token::Plus | Token::Minus | Token::Multiply | Token::Divide | Token::Power => true,
            _ => false,
        }
    }

    fn get_precedence(&self) -> i32 {
        match self {
            Token::Power => 3,
            Token::Multiply | Token::Divide => 2,
            Token::Plus | Token::Minus => 1,
            _ => 0,
        }
    }

    fn get_associative(&self) -> Associative {
        match self {
            Token::Power => Associative::Right,
            _ => Associative::Left,
        }
    }

    fn compute(&self, l: i32, r: i32) -> Option<i32> {
        match self {
            Token::Plus => Some(l + r),
            Token::Minus => Some(l - r),
            Token::Multiply => Some(l * r),
            Token::Divide => Some(l / r),
            Token::Power => Some(l.pow(r as u32)), // this does not currently support negative powers
            _ => None,
        }
    }
}

// parse the expression
// use peekable rather than a usual iterator so we can peek at the next item without consuming it
struct Tokenizer<'a> {
    tokens: Peekable<Chars<'a>>,
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.consume_whitespaces();

        match self.tokens.peek() {
            Some(c) if c.is_numeric() => self.scan_number(), // if we see a number, we don't want to just take it, e.g. 42, we don't want to just take 4 and then take 2
            Some(_) => self.scan_operator(),
            None => return None,
        }
    }
}

impl<'a> Tokenizer<'a> {
    fn new(expr: &'a str) -> Self {
        Self {
            tokens: expr.chars().peekable(),
        }
    }

    fn consume_whitespaces(&mut self) {
        while let Some(&c) = self.tokens.peek() {
            if c.is_whitespace() {
                self.tokens.next();
            } else {
                break;
            }
        }
    }

    fn scan_number(&mut self) -> Option<Token> {
        let mut num = String::new();
        while let Some(&c) = self.tokens.peek() {
            if c.is_numeric() {
                num.push(c);
                self.tokens.next();
            } else {
                break;
            }
        }

        match num.parse() {
            Ok(n) => Some(Token::Number(n)),
            Err(_) => None,
        }
    }

    fn scan_operator(&mut self) -> Option<Token> {
        match self.tokens.next() {
            Some('+') => Some(Token::Plus),
            Some('-') => Some(Token::Minus),
            Some('*') => Some(Token::Multiply),
            Some('/') => Some(Token::Divide),
            Some('^') => Some(Token::Power),
            Some('(') => Some(Token::LeftParenthesis),
            Some(')') => Some(Token::RightParenthesis),
            _ => None,
        }
    }
}

pub struct Expression<'a> {
    // this second layer of Peekable does NOT introduce a second layer of data or a multidimensional array
    // it still holds the same list of Chars
    iter: Peekable<Tokenizer<'a>>,
}

impl<'a> Expression<'a> {
    pub fn new(expr_str: &'a str) -> Self {
        Self {
            iter: Tokenizer::new(expr_str).peekable(),
        }
    }

    /// evaluate atomic expressions
    fn compute_atomic(&mut self) -> Result<i32, ExpressionError> {
        match self.iter.peek() {
            // return if it's a number
            Some(Token::Number(n)) => {
                let val = *n;
                self.iter.next();
                return Ok(val);
            }
            // if it is a left parenthesis, evaluate the entire expression inside
            Some(Token::LeftParenthesis) => {
                self.iter.next();
                let result = self.compute_expression(1)?;
                match self.iter.next() {
                    Some(Token::RightParenthesis) => (),
                    _ => return Err(ExpressionError::Parsing("Unexpected character".into())), // right parenthesis not found, unmatched left parenthesis
                }
                return Ok(result);
            }
            _ => {
                return Err(ExpressionError::Parsing(
                    "Expecting a number or left parenthesis".into(),
                ))
            }
        }
    }

    fn compute_expression(&mut self, min_precedence: i32) -> Result<i32, ExpressionError> {
        // compute the first token
        let mut atom_lhs = self.compute_atomic()?;

        loop {
            let curr_token = self.iter.peek();
            if curr_token.is_none() {
                break; // nothing left to do
            }
            let token = *curr_token.unwrap();

            // new token must be an operator, it would not make sense to have a number after an atomic expression
            // new token's precedence much be largest than min_precedence
            if !token.is_operator() || token.get_precedence() < min_precedence {
                break;
            }

            let mut next_prec = token.get_precedence();
            if token.get_associative() == Associative::Left {
                next_prec += 1;
            }

            // now advance the iterator
            self.iter.next();

            // recursively compute the right hand side
            let atom_rhs = self.compute_expression(next_prec)?;

            // now simply combine left and right
            match token.compute(atom_lhs, atom_rhs) {
                Some(res) => atom_lhs = res,
                None => return Err(ExpressionError::Parsing("Unexpected expr".into())),
            }
        }
        Ok(atom_lhs)
    }

    pub fn eval(&mut self) -> Result<i32, ExpressionError> {
        let result = self.compute_expression(1)?;
        // if there are still tokens left over, then there was a parsing error
        if self.iter.peek().is_some() {
            return Err(ExpressionError::Parsing("Unexpected end of expr".into()));
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_computes() {
        let expr_str = "21 + 3 + 6 * 27 - (92 - 12) / 5 + 24";
        let mut expr_parsed = Expression::new(expr_str);

        let expected_result = 21 + 3 + 6 * 27 - (92 - 12) / 5 + 24; // 194
        assert_eq!(Ok(expected_result), expr_parsed.eval());
    }

    #[test]
    fn expression_error() {
        let expr_str = "9 + + 4";
        let mut expr_parsed = Expression::new(expr_str);
        assert_eq!(
            Err(ExpressionError::Parsing(
                "Expecting a number or left parenthesis".to_string()
            )),
            expr_parsed.eval()
        );
    }
}

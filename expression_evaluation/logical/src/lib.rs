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

// tokens/symbols in an expression
#[derive(Debug, Clone, Copy)]
enum Token {
    True,
    False,
    And,
    Or,
    Implies,
    Converse,
    Equivalent,
    LeftParenthesis,
    RightParenthesis,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fmt = match self {
            Token::True => "T",
            Token::False => "F",
            Token::And => "&",
            Token::Or => "|",
            Token::Implies => ">",
            Token::Converse => "<",
            Token::Equivalent => "=",
            Token::LeftParenthesis => "(",
            Token::RightParenthesis => ")",
        };

        write!(f, "{}", fmt.to_string())
    }
}

impl Token {
    fn is_operator(&self) -> bool {
        match self {
            Token::And | Token::Or | Token::Implies | Token::Converse | Token::Equivalent => true,
            _ => false,
        }
    }

    // precendence rules
    fn get_precedence(&self) -> i32 {
        match self {
            Token::And => 4,
            Token::Or => 3,
            Token::Implies | Token::Converse => 2,
            Token::Equivalent => 1,
            _ => 0,
        }
    }

    fn compute(&self, l: bool, r: bool) -> Option<bool> {
        match self {
            Token::And => Some(l & r),
            Token::Or => Some(l | r),
            Token::Implies => Some(!(l && !r)),
            Token::Converse => Some(!(r && !l)),
            Token::Equivalent => Some(l == r),
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
            Some(_) => self.scan_token(),
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

    fn scan_token(&mut self) -> Option<Token> {
        match self.tokens.next() {
            Some('T') => Some(Token::True),
            Some('F') => Some(Token::False),
            Some('&') => Some(Token::And),
            Some('|') => Some(Token::Or),
            Some('>') => Some(Token::Implies),
            Some('<') => Some(Token::Converse),
            Some('=') => Some(Token::Equivalent),
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
    fn compute_atomic(&mut self) -> Result<bool, ExpressionError> {
        match self.iter.peek() {
            // return if it's a truth value
            Some(Token::True) => {
                self.iter.next();
                return Ok(true);
            }
            Some(Token::False) => {
                self.iter.next();
                return Ok(false);
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
                    "Expecting a truth value or left parenthesis".into(),
                ))
            }
        }
    }

    fn compute_expression(&mut self, min_precedence: i32) -> Result<bool, ExpressionError> {
        // compute the first token
        let mut atom_lhs = self.compute_atomic()?;

        loop {
            let curr_token = self.iter.peek();
            if curr_token.is_none() {
                break; // nothing left to do
            }
            let token = *curr_token.unwrap();

            // new token must be an operator, it would not make sense to have a truth value after an atomic expression
            // new token's precedence much be largest than min_precedence
            if !token.is_operator() || token.get_precedence() < min_precedence {
                break;
            }

            let mut next_prec = token.get_precedence();
            next_prec += 1;

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

    pub fn eval(&mut self) -> Result<bool, ExpressionError> {
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
    fn simple_expression_computes() {
        let expr_str = "T & T";
        let mut expr_parsed = Expression::new(expr_str);
        assert_eq!(Ok(true), expr_parsed.eval());

        let expr_str = "T & F";
        let mut expr_parsed = Expression::new(expr_str);
        assert_eq!(Ok(false), expr_parsed.eval());

        let expr_str = "F | F";
        let mut expr_parsed = Expression::new(expr_str);
        assert_eq!(Ok(false), expr_parsed.eval());

        let expr_str = "T | F";
        let mut expr_parsed = Expression::new(expr_str);
        assert_eq!(Ok(true), expr_parsed.eval());
    }

    #[test]
    fn complex_expression_computes() {
        let expr_str = "((T & F) | (T > F > F)) = (T < F)";
        let mut expr_parsed = Expression::new(expr_str);
        assert_eq!(Ok(true), expr_parsed.eval());
    }

    #[test]
    fn expression_error() {
        let expr_str = "T & | T";
        let mut expr_parsed = Expression::new(expr_str);
        assert_eq!(
            Err(ExpressionError::Parsing(
                "Expecting a truth value or left parenthesis".to_string()
            )),
            expr_parsed.eval()
        );
    }
}

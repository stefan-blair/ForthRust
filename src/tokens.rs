use std::iter;

use super::generic_numbers;

pub struct TokenStream<'a> {
    iterator: Box<(dyn Iterator<Item = Token> + 'a)>,
    cache: Option<Token>
}

impl<'a> TokenStream<'a> {
    pub fn new(iterator: Box<dyn Iterator<Item = Token> + 'a>) -> Self {
        TokenStream {
            iterator,
            cache: None
        }
    }

    pub fn from_string(string: &'a str) -> Self {
        Self::new(Box::new(string.split_ascii_whitespace().map(|s| Token::tokenize(s))))
    }

    pub fn _empty() -> Self {
        TokenStream {
            iterator: Box::new(iter::empty()),
            cache: None
        }
    }

    pub fn next(&mut self) -> Option<Token> {
        self.cache.take().or_else(|| self.iterator.next())
    }

    pub fn _push(&mut self, token: Token) {
        self.cache = Some(token);
    }

    pub fn _with_push(mut self, token: Token) -> Self {
        self._push(token);
        self
    }

    pub fn _peek(&mut self) -> Option<&Token> {
        self.cache = self.next();
        self.cache.as_ref()
    }
}

#[derive(Debug)]
pub enum Token {
    Integer(generic_numbers::Number),
    Name(String),
}

impl Token {
    pub fn tokenize(s: &str) -> Self {
        s.to_uppercase()
            .as_str()
            .parse::<generic_numbers::Number>()
            .map_or_else(|_| Token::Name(s.to_uppercase()), |i| Token::Integer(i))
    }
}

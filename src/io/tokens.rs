use crate::evaluate::Error;
use crate::environment::generic_numbers;


pub struct TokenStream<'a> {
    stream: Box<dyn Iterator<Item = char> + 'a>
}

impl<'a> TokenStream<'a> {
    pub fn new<I: Iterator<Item = char> + 'a>(stream: I) -> Self {
        Self { stream: Box::new(stream) }
    }

    pub fn empty() -> Self {
        Self { stream: Box::new(std::iter::empty()) }
    }

    pub fn next(&mut self) -> Result<Token, Error> {
        let mut s = String::new();
        if let Some(c) = self.stream.find(|c| !c.is_whitespace()) {
            s.push(c);
            while let Some(next_char) = self.stream.next() {
                if next_char.is_whitespace() {
                    break;
                } else {
                    s.push(next_char)
                }
            }

            Ok(Token::tokenize(&s))
        } else {
            Err(Error::NoMoreTokens)
        }
    }

    pub fn next_word(&mut self) -> Result<String, Error> {
        match self.next()? {
            Token::Word(word) => Ok(word),
            _ => Err(Error::InvalidWord)
        }
    }

    pub fn next_char(&mut self) -> Result<char, Error> {
        self.stream.next().ok_or(Error::NoMoreTokens)
    }

    // todo: implement some sort of next_char function
}

#[derive(Debug)]
pub enum Token {
    Integer(generic_numbers::Number),
    Word(String),
}

impl Token {
    pub fn tokenize(s: &str) -> Self {
        fn parse_number(s: &str) -> Option<generic_numbers::Number> {
            s.strip_prefix("0x").and_then(|x| generic_numbers::Number::from_str_radix(x, 16).ok())
            .or_else(|| s.strip_prefix("0b").and_then(|x| generic_numbers::Number::from_str_radix(x, 2).ok()))
            .or_else(|| s.parse::<generic_numbers::Number>().ok())
        }

        parse_number(s).map_or_else(|| Token::Word(s.to_uppercase()), |i| Token::Integer(i))
    }
}

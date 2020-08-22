use crate::environment::generic_numbers;


pub struct TokenStream<'a> {
    stream: Box<dyn Iterator<Item = char> + 'a>
}

impl<'a> TokenStream<'a> {
    pub fn new<I: Iterator<Item = char> + 'a>(stream: I) -> Self {
        Self { stream: Box::new(stream) }
    }

    pub fn next(&mut self) -> Option<Token> {
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

            Some(Token::tokenize(&s))
        } else {
            None
        }
    }

    pub fn next_char(&mut self) -> Option<char> {
        self.stream.next()
    }

    // todo: implement some sort of next_char function
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

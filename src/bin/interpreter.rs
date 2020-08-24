use std::io::{self, Write};

use forth::{Forth, output_stream, debugger};


/**
 * An iterator that continually reads from standard input.
 */
struct StdinStream {
    current_line: String
}

impl StdinStream {
    fn new() -> Self {
        Self { current_line: String::new() }
    }
}

impl Iterator for StdinStream {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_line.pop()
            .or_else(|| match io::stdin().read_line(&mut self.current_line) {
                Ok(_) => {
                    self.current_line = self.current_line.chars().rev().collect();
                    self.current_line.pop()
                }
                Err(_) => None
            })
    }
}

struct StdoutStream {}

impl StdoutStream {
    pub fn new() -> Self {
        Self{}
    }
}

impl output_stream::OutputStream for StdoutStream {
    fn write(&mut self, m: &str) {
        print!("{}", m);
        io::stdout().flush();
    }

    fn writeln(&mut self, m: &str) {
        println!("{}", m); 
    }
}

fn main() {
    let mut forth = Forth::new(StdoutStream::new());
    forth.add_operations(vec![("DEBUG", false, debugger::debug)]);
    let result = forth.eval_stream(StdinStream::new());
    println!("Finished evaluating: {:?}", result);
}
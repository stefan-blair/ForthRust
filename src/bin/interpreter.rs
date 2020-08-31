use std::io::{self, Write};

use forth::{Forth, output_stream, kernels, debugger};


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
    let mut output_stream = StdoutStream::new();
    let mut forth = Forth::<debugger::DebugKernel<kernels::DefaultKernel>>::new();
    forth.state.add_operations(vec![
        ("DEBUG", false, debugger::debug)
    ]);
    // forth.add_operations(vec![("DEBUG", false, debugger::debug)]);
    let result = forth.evaluate_stream(StdinStream::new(), &mut output_stream);
    println!("Finished evaluating: {:?}", result);
}
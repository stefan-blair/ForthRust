use std::io::{self, Write};

use forth::{Forth, output_stream, kernels, debugger, profiler};


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
        assert!(io::stdout().flush().is_ok());
    }

    fn writeln(&mut self, m: &str) {
        println!("{}", m); 
    }
}

fn main() {
    let mut output_stream = StdoutStream::new();
    let mut forth = Forth::<profiler::ProfilerKernel<debugger::DebugKernel<kernels::DefaultKernel>>>::new();
    assert!(forth.evaluate_string(": TEST 1 . 2 . 3 . 4 . 5 . ; debug 0xd0 set_break continue ", &mut output_stream::DropOutputStream::new()).is_ok());
    assert!(forth.evaluate_string(": TESTING 1 2 + + 5 * ;", &mut output_stream::DropOutputStream::new()).is_ok());
    let result = forth.evaluate_stream(StdinStream::new(), &mut output_stream);
    println!("Finished evaluating: {:?}", result);

    forth.kernel.global_information.dump_statistics(&forth.state, &mut output_stream);
}
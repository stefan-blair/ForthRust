use std::io::{self, Write, BufReader, BufRead};
use std::fs::File;

use forth::{Forth, output_stream, kernels, debugger};


struct FileStream {
    current_line: String,
    reader: BufReader<File>,
}

impl FileStream {
    fn new(reader: BufReader<File>) -> Self {
        Self { current_line: String::new(), reader: reader}
    }
}

impl Iterator for FileStream {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_line.pop()
            .or_else(|| match self.reader.read_line(&mut self.current_line) {
                Ok(_) => {
                    self.current_line = self.current_line.chars().rev().collect();
                    self.current_line.pop()
                }
                Err(_) => None
            })   
    }
}

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
    let mut forth = Forth::<debugger::DebugKernel<kernels::DefaultKernel>>::new().with_output_stream(&mut output_stream);
    forth.kernel.init_io(StdinStream::new(), StdoutStream::new());
 
    let file_path = std::env::args().nth(1).expect("Please provide an input path");
    let file = File::open(file_path).expect("File not found / able to be opened");
    assert!(Ok(()) == forth.evaluate_stream(FileStream::new(BufReader::new(file))));

    assert!(Ok(()) == forth.evaluate_stream(StdinStream::new()));
 }
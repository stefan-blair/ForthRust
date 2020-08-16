use std::mem;


pub struct OutputStream {
    stream: String
}

impl OutputStream {
    pub fn new() -> Self {
        OutputStream {
            stream: String::new()
        }
    }

    pub fn write(&mut self, m: &str) {
        self.stream.push_str(m);
    }

    pub fn writeln(&mut self, m: &str) {
        self.stream.push_str(m);
        self.stream.push('\n');
    }

    pub fn consume(&mut self) -> String {
        mem::take(&mut self.stream)
    }
}
use std::mem;


pub trait OutputStream {
    fn write(&mut self, m: &str);
    fn writeln(&mut self, m: &str);
}

pub struct BufferedOutputStream {
    stream: String
}

impl BufferedOutputStream {
    pub fn new() -> Self {
        BufferedOutputStream {
            stream: String::new()
        }
    }

    pub fn consume(&mut self) -> String {
        mem::take(&mut self.stream)
    }
}

impl OutputStream for BufferedOutputStream {
    fn write(&mut self, m: &str) {
        self.stream.push_str(m);
    }

    fn writeln(&mut self, m: &str) {
        self.stream.push_str(m);
        self.stream.push('\n');
    }
}

pub struct DropOutputStream {}

impl DropOutputStream {
    pub fn new() -> Self {
        DropOutputStream {}
    }
}

impl OutputStream for DropOutputStream {
    fn write(&mut self, _m: &str) {}
    fn writeln(&mut self, _m: &str) {}
}
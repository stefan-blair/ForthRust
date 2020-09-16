use std::mem;


pub trait OutputStream {
    fn write(&mut self, m: &str);
    fn writeln(&mut self, m: &str) {
        self.write(m);
        self.write("\n");
    }
    fn consume(&mut self) -> String {
        return "".to_string();
    }
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
}

impl OutputStream for BufferedOutputStream {
    fn write(&mut self, m: &str) {
        self.stream.push_str(m);
    }

    
    fn consume(&mut self) -> String {
        mem::take(&mut self.stream)
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
}

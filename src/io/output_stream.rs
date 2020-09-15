use std::mem;


pub trait OutputStream {
    fn write(&mut self, m: &str);
    fn writeln(&mut self, m: &str) {
        self.write(m);
        self.write("\n");
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

    pub fn consume(&mut self) -> String {
        mem::take(&mut self.stream)
    }
}

impl OutputStream for BufferedOutputStream {
    fn write(&mut self, m: &str) {
        self.stream.push_str(m);
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

pub struct OptionalOutputStream<'a, 'b>(Option<&'a mut (dyn OutputStream + 'b)>);

impl<'a, 'b> OptionalOutputStream<'a, 'b> {
    pub fn empty() -> Self {
        Self(None)
    }

    pub fn with(output_stream: &'a mut (dyn OutputStream + 'b)) -> Self {
        Self(Some(output_stream))
    }
}

impl<'a, 'b> OutputStream for OptionalOutputStream<'a, 'b> {
    fn write(&mut self, m: &str) {
        if let Some(output_stream) = self.0.as_mut() {
            output_stream.write(m)
        }
    }
}

use std::fmt::Display;
use std::io::{self, Write};
use std::marker::PhantomData;
use std::str::FromStr;

/// An iterator that lazy reads from the standard input.
#[derive(Debug)]
#[must_use]
pub struct Stdin<T, P: Fn(usize)> {
    /// [`read_line`](io::Stdin::read_line) buffer
    buf: String,

    /// The number of elements already returned.
    i: usize,

    /// The prompt is called with the number of elements already returned.
    prompt: P,

    _phantom: PhantomData<T>,
}

impl<T, P: Fn(usize)> Stdin<T, P> {
    pub fn new(prompt: P) -> Stdin<T, P> {
        Stdin {
            buf: String::new(),
            i: 0,
            prompt,
            _phantom: PhantomData,
        }
    }
}

impl<T: FromStr<Err: Display>, P: Fn(usize)> Iterator for Stdin<T, P> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            (self.prompt)(self.i);
            _ = io::stdout().flush();

            self.buf.clear();
            match io::stdin().read_line(&mut self.buf) {
                Ok(0) => {
                    println!("<eof>");
                    return None
                },
                Ok(_) => {}
                Err(e) => {
                    eprintln!("error: failed to read stdin: {e}");
                    return None;
                }
            };
            
            match T::from_str(self.buf.trim()) {
                Ok(v) => {
                    self.i += 1;
                    break Some(v)
                },
                Err(_) if self.buf.trim_ascii_end().is_empty() => return None,
                Err(e) => eprintln!("error: {e}"),
            }
        }
    }
}

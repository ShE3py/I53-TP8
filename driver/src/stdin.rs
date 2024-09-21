use rame::model::Instruction;
use std::any::TypeId;
use std::fmt::Display;
use std::io;
use std::io::Write;
use std::marker::PhantomData;
use std::str::FromStr;

/// An iterator that lazy reads from the standard input.
#[derive(Debug)]
#[must_use]
pub struct Stdin<T> {
    buf: String,
    i: usize,
    _phantom: PhantomData<T>,
}

impl<T> Stdin<T> {
    pub fn new() -> Stdin<T> {
        Stdin {
            buf: String::new(),
            i: 0,
            _phantom: PhantomData,
        }
    }
}

impl<T: FromStr<Err: Display> + 'static> Iterator for Stdin<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match TypeId::of::<T>() {
                ty if [
                    TypeId::of::<Instruction<i8>>(),
                    TypeId::of::<Instruction<i16>>(),
                    TypeId::of::<Instruction<i32>>(),
                    TypeId::of::<Instruction<i64>>(),
                    TypeId::of::<Instruction<i128>>(),
                ].contains(&ty) => print!("{} | ", self.i),
                
                ty if [
                    TypeId::of::<i8>(),
                    TypeId::of::<i16>(),
                    TypeId::of::<i32>(),
                    TypeId::of::<i64>(),
                    TypeId::of::<i128>(),
                ].contains(&ty) => print!("E{} = ", self.i),
                
                _ => unimplemented!(),
            }
            
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

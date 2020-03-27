use crate::{chunk::Chunk, scanner::Scanner, CedarError};

pub fn compile(source: String) -> Result<Chunk, CedarError> {
  let mut scanner = Scanner::new(source);
  let tokens = scanner.scan()?;
  println!("{:#?}", tokens);
  todo!()
}

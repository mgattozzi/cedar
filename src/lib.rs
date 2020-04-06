pub mod chunk;
pub mod compiler;
pub mod libstd;
pub mod native;
pub mod scanner;
pub mod value;
pub mod vm;

pub use chunk::ChunkError;
pub use compiler::CompilerError;
use scanner::ScannerError;
use std::{fmt, io, num::ParseFloatError};
pub use vm::{InterpreterResult, VM};

#[derive(Debug)]
pub enum CedarError {
  InterpreterResult(InterpreterResult),
  Io(io::Error),
  ScannerError(ScannerError),
  CompilerError(CompilerError),
  ParseFloatError(ParseFloatError),
  ChunkError(ChunkError),
}
impl From<io::Error> for CedarError {
  fn from(e: io::Error) -> CedarError {
    CedarError::Io(e)
  }
}
impl From<InterpreterResult> for CedarError {
  fn from(e: InterpreterResult) -> CedarError {
    CedarError::InterpreterResult(e)
  }
}
impl From<ScannerError> for CedarError {
  fn from(e: ScannerError) -> CedarError {
    CedarError::ScannerError(e)
  }
}
impl From<CompilerError> for CedarError {
  fn from(e: CompilerError) -> CedarError {
    CedarError::CompilerError(e)
  }
}
impl From<ParseFloatError> for CedarError {
  fn from(e: ParseFloatError) -> CedarError {
    CedarError::ParseFloatError(e)
  }
}
impl From<ChunkError> for CedarError {
  fn from(e: ChunkError) -> CedarError {
    CedarError::ChunkError(e)
  }
}

impl fmt::Display for CedarError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      CedarError::InterpreterResult(e) => write!(f, "{}", e),
      CedarError::Io(e) => write!(f, "{}", e),
      CedarError::ScannerError(e) => write!(f, "{}", e),
      CedarError::CompilerError(e) => write!(f, "{}", e),
      CedarError::ParseFloatError(e) => write!(f, "{}", e),
      CedarError::ChunkError(e) => write!(f, "{}", e),
    }
  }
}

impl std::error::Error for CedarError {}

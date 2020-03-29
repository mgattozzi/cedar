pub mod chunk;
pub mod compiler;
pub mod scanner;
pub mod value;
pub mod vm;

use chunk::ChunkError;
use compiler::CompilerError;
use scanner::ScannerError;
use std::{
  env, fmt, fs,
  io::{self, Write},
  num::ParseFloatError,
  path::PathBuf,
  process::exit,
};
use vm::{InterpretResult, VM};

pub fn main() {
  let args = env::args();
  let res = if args.len() > 2 {
    println!("Usage: cedar [script]");
    exit(64);
  } else if let Some(arg) = args.skip(1).next() {
    run_file(arg.into())
  } else {
    repl()
  };

  if let Err(e) = res {
    eprintln!("{}", e);
    match e {
      CedarError::InterpretResult(i) => match i {
        InterpretResult::CompileError => exit(65),
        InterpretResult::RuntimeError(_, _) => exit(70),
      },
      _ => exit(64),
    }
  }
}

fn run_file(path: PathBuf) -> Result<(), CedarError> {
  run(fs::read_to_string(&path)?)
}

fn repl() -> Result<(), CedarError> {
  let stdin = io::stdin();
  let mut stdout = io::stdout();
  loop {
    print!("> ");
    stdout.flush()?;
    let mut line = String::new();
    stdin.read_line(&mut line)?;
    stdout.flush()?;
    if let Err(e) = run(line) {
      eprintln!("{}", e);
    }
  }
}

fn run(source: String) -> Result<(), CedarError> {
  let mut vm = VM::new();
  vm.interpret(source)
}

#[derive(Debug)]
pub enum CedarError {
  InterpretResult(InterpretResult),
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
impl From<InterpretResult> for CedarError {
  fn from(e: InterpretResult) -> CedarError {
    CedarError::InterpretResult(e)
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
      CedarError::InterpretResult(e) => write!(f, "{}", e),
      CedarError::Io(e) => write!(f, "{}", e),
      CedarError::ScannerError(e) => write!(f, "{}", e),
      CedarError::CompilerError(e) => write!(f, "{}", e),
      CedarError::ParseFloatError(e) => write!(f, "{}", e),
      CedarError::ChunkError(e) => write!(f, "{}", e),
    }
  }
}

impl std::error::Error for CedarError {}

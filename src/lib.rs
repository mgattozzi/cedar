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
use vm::{InterpreterResult, VM};

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
    match e {
      CedarError::CompilerError(c) => match c {
        CompilerError::Failed => exit(64),
        _ => unreachable!(),
      },
      CedarError::InterpreterResult(i) => match i {
        InterpreterResult::CompileError(_) => {
          eprintln!("{}", i);
          exit(65);
        }
        InterpreterResult::RuntimeError(_, _) => {
          eprintln!("{}", i);
          exit(70);
        }
      },
      _ => {
        eprintln!("{}", e);
        exit(64);
      }
    }
  }
}

fn run_file(path: PathBuf) -> Result<(), CedarError> {
  let mut vm = VM::new();
  run(&mut vm, fs::read_to_string(&path)?)
}

fn repl() -> Result<(), CedarError> {
  let stdin = io::stdin();
  let mut stdout = io::stdout();
  let mut vm = VM::new();
  loop {
    print!("> ");
    stdout.flush()?;
    let mut line = String::new();
    stdin.read_line(&mut line)?;
    stdout.flush()?;
    if let Err(e) = run(&mut vm, line) {
      eprintln!("{}", e);
    }
  }
}

fn run(vm: &mut VM, source: String) -> Result<(), CedarError> {
  vm.interpret(source)
}

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

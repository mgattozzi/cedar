use cedar::{CedarError, CompilerError, InterpreterResult, VM};
use rustyline::{error::ReadlineError, Editor};
use std::{env, fs, path::PathBuf, process::exit};
fn main() {
  let mut args = env::args();
  let res = if args.len() > 2 {
    println!("Usage: cedar [script]");
    exit(64);
  } else if let Some(arg) = args.nth(1) {
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
  let mut vm = VM::new();
  let mut rl = Editor::<()>::new();
  loop {
    let readline = rl.readline(">> ");
    match readline {
      Ok(line) => match line.trim() {
        "exit" | "quit" | "q" => break Ok(()),
        _ => {
          rl.add_history_entry(line.as_str());
          if let Err(e) = run(&mut vm, line) {
            eprintln!("{}", e);
          }
        }
      },
      Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break Ok(()),
      Err(e) => {
        eprintln!("{}", e);
        exit(64);
      }
    }
  }
}

fn run(vm: &mut VM, source: String) -> Result<(), CedarError> {
  vm.interpret(source)
}

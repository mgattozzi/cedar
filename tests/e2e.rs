use assert_cmd::Command;
use pretty_assertions::assert_eq;
use std::{error::Error, path::PathBuf};

#[test]
fn control_flow() -> Result<(), Box<dyn Error>> {
  let path = PathBuf::from("tests")
    .join("cedar-scripts")
    .join("control-flow.cdr");
  let mut cmd = Command::cargo_bin("cedarc")?;
  cmd.arg(path);
  cmd.assert().success();
  let stdout = String::from_utf8(cmd.output()?.stdout)?;
  assert_eq!(stdout, CONTROL_FLOW);

  Ok(())
}

#[test]
fn scopes() -> Result<(), Box<dyn Error>> {
  let path = PathBuf::from("tests")
    .join("cedar-scripts")
    .join("scopes.cdr");
  let mut cmd = Command::cargo_bin("cedarc")?;
  cmd.arg(path);
  cmd.assert().success();
  let stdout = String::from_utf8(cmd.output()?.stdout)?;
  assert_eq!(stdout, SCOPES);

  Ok(())
}

#[test]
fn functions() -> Result<(), Box<dyn Error>> {
  let path = PathBuf::from("tests")
    .join("cedar-scripts")
    .join("functions.cdr");
  let mut cmd = Command::cargo_bin("cedarc")?;
  cmd.arg(path);
  cmd.assert().success();
  let stdout = String::from_utf8(cmd.output()?.stdout)?;
  assert_eq!(stdout, FUNCTIONS);

  Ok(())
}

#[test]
fn native_functions() -> Result<(), Box<dyn Error>> {
  let path = PathBuf::from("tests")
    .join("cedar-scripts")
    .join("native.cdr");
  let mut cmd = Command::cargo_bin("cedarc")?;
  cmd.arg(path);
  cmd.assert().success();
  let stdout = String::from_utf8(cmd.output()?.stdout)?;
  assert_eq!(stdout, NATIVE);
  std::fs::remove_file("test-file")?;
  Ok(())
}

const NATIVE: &str = r#"Testing writes
"#;
const FUNCTIONS: &str = r#"Hello
It works!
a^2 + b^2 = c^2 is
true
Look at ME I am the captain now
true
"#;
const SCOPES: &str = r#"3
2
3
0
global
"#;
const CONTROL_FLOW: &str = r#"When you get older it's harder to make mistakes, as you get more responsibilities the less mistakes you are allowed to make.
Being an adult is hard for everybody. That's what alcohol is for.
That looks pretty good. Now all your secret admirers won't be able to resist you.
--Like I'd have any.
You have to think you do. That's the secret to being pretty.
You're gonna carry that weight.
Don't believe in yourself. Believe in me! Believe in the Kamina who believes in you!
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Alright
Owari da
"#;

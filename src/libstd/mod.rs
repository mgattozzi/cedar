use crate::{native::NativeFuncHolder, value::Value};
use std::{borrow::Cow, collections::HashMap, rc::Rc};

pub mod io;

use self::io::*;

pub fn load() -> HashMap<Cow<'static, str>, Value> {
  let mut std = HashMap::new();
  let read: fn(Cow<'static, str>) -> Cow<'static, str> = read_file;
  std.insert(
    "read-file".into(),
    Value::NativeFn(NativeFuncHolder {
      inner: Rc::new(read),
    }),
  );
  let write: fn(Cow<'static, str>, Cow<'static, str>) -> () = write_file;
  std.insert(
    "write-file".into(),
    Value::NativeFn(NativeFuncHolder {
      inner: Rc::new(write),
    }),
  );

  std
}

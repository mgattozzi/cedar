use std::{borrow::Cow, fs, path::PathBuf};

pub fn read_file(path: Cow<'static, str>) -> Cow<'static, str> {
  fs::read_to_string(&PathBuf::from(&*path)).unwrap().into()
}
pub fn write_file(path: Cow<'static, str>, content: Cow<'static, str>) {
  fs::write(&PathBuf::from(&*path), &*content).unwrap();
}

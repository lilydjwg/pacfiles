use std::process::{Command, Stdio, Child, ChildStdout};
use std::io::{BufReader, BufRead, Result as IoResult};
use std::ffi::OsStr;
use std::path::Path;

use tracing::{debug, error};

pub struct Plocate {
  process: Child,
  bufreader: BufReader<ChildStdout>,
}

pub struct PackageFile {
  line: String,
  version_start: usize,
  filepath_start: usize,
}

impl Plocate {
  pub fn new(db: &str, pattern: &str, regex: bool, basename: bool) -> IoResult<Self> {
    debug!("Plocate::new({db}, {pattern}, {regex}, {basename}");
    let mut cmd = Command::new("plocate");
    if regex {
      cmd.arg("--regex");
    }
    if basename {
      cmd.arg("-b");
    }
    let mut process = cmd
      .args(["-d", db, "--", pattern])
      .stdout(Stdio::piped())
      .spawn()?;
    let bufreader = BufReader::new(process.stdout.take().unwrap());
    Ok(Plocate { process, bufreader })
  }
}

impl Iterator for Plocate {
  type Item = IoResult<PackageFile>;
  fn next(&mut self) -> Option<Self::Item> {
    let mut line = String::new();
    match self.bufreader.read_line(&mut line) {
      Ok(0) => {
        match self.process.wait() {
          Ok(st) => {
            if !st.success() && st.code() != Some(1) {
              // exit 1 => not found
              error!("plocate exited with error: {}", st);
            }
          }
          Err(e) => {
            error!("failed to wait plocate: {:?}", e);
          }
        }
        None
      }
      Ok(_) => {
        if line.ends_with('\n') {
          line.pop();
        }
        Some(Ok(PackageFile::new(line)))
      }
      Err(e) => Some(Err(e)),
    }
  }
}

impl PackageFile {
  pub fn new(line: String) -> Self {
    let (pkgpart, _filepath) = line.split_once('/').unwrap();
    let filepath_start = pkgpart.len() + 1;

    debug!("line: {}", line);
    let mut it = pkgpart.rsplitn(3, '-');
    it.next().unwrap(); // pkgrel
    it.next().unwrap(); // pkgver
    let pkgname = it.next().unwrap();
    let version_start = pkgname.len() + 1;

    Self { line, version_start, filepath_start }
  }

  pub fn pkgname(&self) -> &str {
    &self.line[..self.version_start-1]
  }

  pub fn version(&self) -> &str {
    &self.line[self.version_start..self.filepath_start-1]
  }

  pub fn path(&self) -> &str {
    &self.line[self.filepath_start..]
  }
}

#[cfg(test)]
mod test {
  #[test]
  fn test_package_file() {
    let pf = super::PackageFile::new(String::from("vi-1:070224-6/usr/bin/vi"));
    assert_eq!(pf.pkgname(), "vi");
    assert_eq!(pf.version(), "1:070224-6");
    assert_eq!(pf.path(), "usr/bin/vi");
  }
}

pub fn foreach_database(mut f: impl FnMut(&Path) -> IoResult<()>) -> IoResult<()> {
  for entry in std::fs::read_dir("/var/lib/pacman/sync")? {
    let entry = entry?;
    let path = entry.path();

    if path.extension() != Some(OsStr::new("pacfiles")) {
      continue;
    }

    f(&path)?;
  }
  Ok(())
}

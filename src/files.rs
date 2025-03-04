// Copyright (C) 2025 lilydjwg
// 
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
// 
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// 
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::process::{Command, Stdio, Child, ChildStdout};
use std::io::{BufReader, BufRead, Result as IoResult};

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
      .args(["-d", db, "-N", "--", pattern])
      .stdout(Stdio::piped())
      .stderr(Stdio::null())
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

pub fn foreach_database(mut f: impl FnMut(String) -> IoResult<()>) -> IoResult<()> {
  let output = Command::new("pacman-conf")
    .arg("-l")
    .stdout(Stdio::piped())
    .output()?;
  let repos = String::from_utf8(output.stdout).unwrap();
  for repo in repos.split_terminator('\n') {
    let path = format!("/var/lib/pacman/sync/{repo}.pacfiles");
    f(path)?;
  }
  Ok(())
}

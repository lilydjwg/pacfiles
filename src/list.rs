use std::path::Path;
use std::io::{stdout, Write, Result as IoResult};

use eyre::Result;
use nu_ansi_term::Style;

use crate::files;

pub fn list_packages(
  path: &Path,
  _repo: &str,
  packages: &[String],
) -> Result<()> {
  for pkg in files::FilesReader::new(path)? {
    let (pkgname, files) = pkg?;
    if packages.contains(&pkgname) {
      list_package_files(&pkgname, &files)?;
    }
  }
  Ok(())
}

fn list_package_files(pkgname: &str, files: &str) -> IoResult<()> {
  let mut stdout = stdout().lock();
  for file in files::FilesIter::new(files) {
    writeln!(stdout, "{} {}", Style::new().bold().paint(pkgname), file)?;
  }
  Ok(())
}

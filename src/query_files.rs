use std::path::Path;
use std::io::{stdout, Write};

use eyre::Result;
use nu_ansi_term::Color::{Magenta, White};

use crate::files;
use crate::Matcher;

pub fn query_files(
  path: &Path,
  repo: &str,
  matcher: &Matcher,
) -> Result<()> {
  let mut stdout = stdout().lock();
  for pkg in files::FilesReader::new(path)? {
    let (pkgname, files) = pkg?;
    let mut matched = false;
    for file in files::FilesIter::new(&files) {
      if matcher(file) {
        if !matched {
          writeln!(stdout, "{}{}{}",
            Magenta.bold().paint(repo),
            Magenta.bold().paint("/"),
            White.bold().paint(&pkgname),
          )?;
          matched = true;
        }
        writeln!(stdout, "    {}", file)?;
      }
    }
  }
  Ok(())
}

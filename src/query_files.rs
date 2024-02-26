use std::path::Path;
use std::io::{stdout, Write};

use eyre::Result;
use nu_ansi_term::Color::{Magenta, White};

use crate::files;
use crate::Matcher;

pub fn query_files<'a, 'b: 'a>(
  path: &'b Path,
  repo: &'b str,
  matcher: &'b Matcher,
  scope: &'a scoped_thread_pool::Scope<'b>,
) -> Result<()> {
  for pkg in files::FilesReader::new(path)? {
    let (pkgname, files) = pkg?;
    scope.execute(move || {
      let mut matched = false;
      for file in files::FilesIter::new(&files) {
        if matcher(file) {
          let mut stdout = stdout().lock();
          if !matched {
            writeln!(stdout, "{}{}{}",
              Magenta.bold().paint(repo),
              Magenta.bold().paint("/"),
              White.bold().paint(&pkgname),
            ).unwrap();
            matched = true;
          }
          writeln!(stdout, "    {}", file).unwrap();
        }
      }
    });
  }
  Ok(())
}

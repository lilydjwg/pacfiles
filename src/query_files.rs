use std::io::{stdout, StdoutLock, Write, Result as IoResult};

use nu_ansi_term::{Style, Color};

use crate::files;

pub fn query_files(queries: &[String], regex: bool, quiet: bool) -> IoResult<()> {
  for query in queries {
    if regex {
      query_files_regex(query, quiet)?;
    } else {
      query_files_pattern(query, quiet)?;
    }
  }
  Ok(())
}

fn query_files_regex(pattern: &str, quiet: bool) -> IoResult<()> {
  let mut stdout = stdout().lock();
  files::foreach_database(|path| {
    let plocate = files::Plocate::new(path.to_str().unwrap(), pattern, true, !pattern.contains('/'))?;
    output_plocate(&mut stdout, plocate, path.file_stem().unwrap().to_str().unwrap(), quiet)
  })
}

fn query_files_pattern(pattern: &str, quiet: bool) -> IoResult<()> {
  let mut stdout = stdout().lock();
  let mut modified_pattern = String::new();
  let p = if !pattern.contains(['*', '?', '[', ']']) {
    modified_pattern.push('[');
    modified_pattern.push(pattern.chars().next().unwrap());
    modified_pattern.push(']');
    modified_pattern.push_str(&pattern[1..]);
    modified_pattern.as_str()
  } else {
    pattern
  };
  files::foreach_database(|path| {
    let plocate = files::Plocate::new(path.to_str().unwrap(), p, false, !pattern.contains('/'))?;
    output_plocate(&mut stdout, plocate, path.file_stem().unwrap().to_str().unwrap(), quiet)
  })
}

fn output_plocate(
  stdout: &mut StdoutLock,
  plocate: files::Plocate,
  repo: &str,
  quiet: bool,
) -> IoResult<()> {
  let mut last_pkgname = String::new();
  for pf in plocate {
    let pf = pf?;
    let pkgname = pf.pkgname();
    let same_pkgname = last_pkgname == pkgname;
    if same_pkgname && quiet {
      continue;
    }
    if quiet {
      writeln!(stdout, "{}/{}", repo, pkgname)?;
    } else {
      let path = pf.path();
      if !same_pkgname {
        let version = pf.version();
        writeln!(stdout, "{}{} {}",
          Color::Magenta.bold().paint(format!("{repo}/")),
          Style::new().bold().paint(pkgname),
          Color::Green.bold().paint(version),
        )?;
      }
      writeln!(stdout, "    {}", path)?;
    }
    if !same_pkgname {
      last_pkgname.clear();
      last_pkgname.push_str(pkgname)
    }
  }
  Ok(())
}

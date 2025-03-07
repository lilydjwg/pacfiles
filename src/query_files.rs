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

use std::io::{stdout, StdoutLock, Write, Result as IoResult};
use std::path::Path;
use std::borrow::Cow;

use nu_ansi_term::{Style, Color};

use crate::files;
use crate::installed::InstalledPackages;

pub fn query_files(queries: &[String], regex: bool, quiet: bool) -> IoResult<()> {
  let installed = InstalledPackages::new()?;
  for query in queries {
    if regex {
      query_files_regex(query, quiet, &installed)?;
    } else {
      query_files_pattern(query, quiet, &installed)?;
    }
  }
  Ok(())
}

fn query_files_regex(
  pattern: &str,
  quiet: bool,
  installed: &InstalledPackages,
) -> IoResult<()> {
  let mut stdout = stdout().lock();
  let is_fullpath = pattern.contains('/');
  let mut found = false;
  files::foreach_database(|path| {
    let plocate = files::Plocate::new(&path, pattern, true, !is_fullpath)?;
    found = output_plocate(
      &mut stdout,
      plocate,
      Path::new(&path).file_stem().unwrap().to_str().unwrap(),
      quiet,
      installed,
      None,
    )? || found;
    Ok(())
  })?;

  if !found {
    std::process::exit(1);
  }

  Ok(())
}

fn query_files_pattern(
  pattern: &str,
  quiet: bool,
  installed: &InstalledPackages,
) -> IoResult<()> {
  let mut stdout = stdout().lock();
  let is_fullpath = pattern.contains('/');
  let is_glob = pattern.contains(['*', '?', '[', ']']);
  let mut validating_path = None;
  let mut modified_pattern = String::new();
  let p = if !is_fullpath && !is_glob {
    modified_pattern.push('[');
    modified_pattern.push(pattern.chars().next().unwrap());
    modified_pattern.push(']');
    modified_pattern.push_str(&pattern[1..]);
    modified_pattern.as_str()
  } else if is_fullpath {
    modified_pattern.push_str("*/");
    if let Some(stripped) = pattern.strip_prefix('/') {
      modified_pattern.push_str(stripped);
      validating_path = Some(stripped);
    } else {
      modified_pattern.push_str(pattern);
      validating_path = Some(pattern);
    }
    modified_pattern.as_str()
  } else {
    pattern
  };
  let mut found = false;
  files::foreach_database(|path| {
    let plocate = files::Plocate::new(&path, p, false, !is_fullpath)?;
    found = output_plocate(
      &mut stdout,
      plocate,
      Path::new(&path).file_stem().unwrap().to_str().unwrap(),
      quiet,
      installed,
      validating_path,
    )? || found;
    Ok(())
  })?;

  if !found {
    std::process::exit(1);
  }

  Ok(())
}

fn output_plocate(
  stdout: &mut StdoutLock,
  plocate: files::Plocate,
  repo: &str,
  quiet: bool,
  installed: &InstalledPackages,
  validating_path: Option<&str>,
) -> IoResult<bool> {
  let mut last_pkgname = String::new();
  let mut found = false;
  for pf in plocate {
    let pf = pf?;
    let pkgname = pf.pkgname();
    let same_pkgname = last_pkgname == pkgname;
    if same_pkgname && quiet {
      continue;
    }
    let path = pf.path();
    if let Some(p) = validating_path {
      if !path.starts_with(p) {
        continue;
      }
    }
    if quiet {
      writeln!(stdout, "{}/{}", repo, pkgname)?;
    } else {
      let version = pf.version();
      if validating_path.is_some() {
        writeln!(stdout, "{} is owned by {}{} {}",
          path,
          Color::Magenta.bold().paint(format!("{repo}/")),
          Style::new().bold().paint(pkgname),
          Color::Green.bold().paint(version),
        )?;
      } else {
        if !same_pkgname {
          let installed_version = installed.package_version(pkgname);
          if let Some(iv) = installed_version {
            let installed_text = if iv == version {
              Cow::Borrowed("[installed]")
            } else {
              Cow::Owned(format!("[installed: {iv}]"))
            };
            writeln!(stdout, "{}{} {} {}",
              Color::Magenta.bold().paint(format!("{repo}/")),
              Style::new().bold().paint(pkgname),
              Color::Green.bold().paint(version),
              Color::Cyan.bold().paint(installed_text),
            )?;
          } else {
            writeln!(stdout, "{}{} {}",
              Color::Magenta.bold().paint(format!("{repo}/")),
              Style::new().bold().paint(pkgname),
              Color::Green.bold().paint(version),
            )?;
          }
        }
        writeln!(stdout, "    {}", path)?;
      }
    }
    if !same_pkgname {
      last_pkgname.clear();
      last_pkgname.push_str(pkgname)
    }
    found = true;
  }
  Ok(found)
}

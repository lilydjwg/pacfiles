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

use nu_ansi_term::{Style, Color};

use crate::files;

pub fn list_packages(packages: &[String], quiet: bool) -> IoResult<()> {
  let mut found = false;
  for pkg in packages {
    found = list_repo_package_files(pkg, quiet)? || found;
  }

  if !found {
    std::process::exit(1);
  }

  Ok(())
}

fn list_repo_package_files(pkg: &str, quiet: bool) -> IoResult<bool> {
  let (repo, pkgname) = if let Some((r, pkgname)) = pkg.split_once('/') {
    (Some(r), pkgname)
  } else {
    (None, pkg)
  };

  let mut found = false;
  let pattern = format!("{}-*", pkgname);
  let mut stdout = stdout().lock();
  if let Some(repo) = repo {
    let path = format!("/var/lib/pacman/sync/{}.pacfiles", repo);
    let plocate = files::Plocate::new(&path, &pattern, false, false)?;
    found = output_plocate(&mut stdout, plocate, pkgname, quiet)? || found;
  } else {
    files::foreach_database(|path| {
      let plocate = files::Plocate::new(&path, &pattern, false, false)?;
      found = output_plocate(&mut stdout, plocate, pkgname, quiet)? || found;
      Ok(())
    })?;
  }
  if !found {
    eprintln!("{} package '{}' was not found",
      Color::Red.bold().paint("error:"),
      pkg,
    );
  }
  Ok(found)
}

fn output_plocate(
  stdout: &mut StdoutLock,
  plocate: files::Plocate,
  pkgname: &str,
  quiet: bool,
) -> IoResult<bool> {
  let mut found = false;
  for pf in plocate {
    let pf = pf?;
    let real_pkgname = pf.pkgname();
    if real_pkgname != pkgname {
      continue;
    }
    let path = pf.path();
    if quiet {
      writeln!(stdout, "{}", path)?;
    } else {
      writeln!(stdout, "{} {}", Style::new().bold().paint(pkgname), path)?;
    }
    found = true;
  }
  Ok(found)
}

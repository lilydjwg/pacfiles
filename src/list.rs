use std::io::{stdout, StdoutLock, Write, Result as IoResult};

use nu_ansi_term::Style;

use crate::files;

pub fn list_packages(packages: &[String], quiet: bool) -> IoResult<()> {
  for pkg in packages {
    let (repo, pkgname) = if let Some((r, pkgname)) = pkg.split_once('/') {
      (Some(r), pkgname)
    } else {
      (None, &pkg[..])
    };
    list_repo_package_files(repo, pkgname, quiet)?;
  }
  Ok(())
}

fn list_repo_package_files(repo: Option<&str>, pkgname: &str, quiet: bool) -> IoResult<()> {
  let pattern = format!("{}-*", pkgname);
  let mut stdout = stdout().lock();
  if let Some(repo) = repo {
    let path = format!("/var/lib/pacman/sync/{}.pacfiles", repo);
    let plocate = files::Plocate::new(&path, &pattern, false, false)?;
    output_plocate(&mut stdout, plocate, pkgname, quiet)?;
  } else {
    files::foreach_database(|path| {
      let plocate = files::Plocate::new(path.to_str().unwrap(), &pattern, false, false)?;
      output_plocate(&mut stdout, plocate, pkgname, quiet)
    })?;
  }
  Ok(())
}

fn output_plocate(
  stdout: &mut StdoutLock,
  plocate: files::Plocate,
  pkgname: &str,
  quiet: bool,
) -> IoResult<()> {
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
  }
  Ok(())
}

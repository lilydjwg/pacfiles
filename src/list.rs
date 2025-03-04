use std::io::{stdout, StdoutLock, Write, Result as IoResult};
use std::ffi::OsStr;

use eyre::Result;
use nu_ansi_term::Style;

use crate::files;

pub fn list_packages(packages: &[String]) -> Result<()> {
  for pkg in packages {
    let (repo, pkgname) = if let Some((r, pkgname)) = pkg.split_once('/') {
      (Some(r), pkgname)
    } else {
      (None, &pkg[..])
    };
    list_repo_package_files(repo, pkgname)?;
  }
  Ok(())
}

fn list_repo_package_files(repo: Option<&str>, pkgname: &str) -> IoResult<()> {
  let pattern = format!("{}-*", pkgname);
  let mut stdout = stdout().lock();
  if let Some(repo) = repo {
    let path = format!("/var/lib/pacman/sync/{}.pacfiles", repo);
    let plocate = files::Plocate::new(&path, &pattern)?;
    output_plocate(&mut stdout, plocate, pkgname)?;
  } else {
    for entry in std::fs::read_dir("/var/lib/pacman/sync")? {
      let entry = entry?;
      let path = entry.path();

      if path.extension() != Some(OsStr::new("pacfiles")) {
        continue;
      }

      let plocate = files::Plocate::new(path.to_str().unwrap(), &pattern)?;
      output_plocate(&mut stdout, plocate, pkgname)?;
    }
  }
  Ok(())
}

fn output_plocate(stdout: &mut StdoutLock, plocate: files::Plocate, pkgname: &str) -> IoResult<()> {
  for pf in plocate {
    let pf = pf?;
    let real_pkgname = pf.pkgname();
    if real_pkgname != pkgname {
      continue;
    }
    let path = pf.path();
    writeln!(stdout, "{} {}", Style::new().bold().paint(pkgname), path)?;
  }
  Ok(())
}

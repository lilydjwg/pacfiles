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

use std::ffi::OsStr;
use std::process::Command;
use std::path::PathBuf;
use std::fs::{File, self};
use std::os::unix::fs::PermissionsExt;
use std::io::{BufReader, Write, BufWriter, ErrorKind};

use eyre::Result;
use tracing::{info, error};
use compress_tools::ArchiveContents;

pub fn refresh(force: bool) -> Result<()> {
  info!("running pacman command");
  let mut child = Command::new("pacman")
    .arg(if force { "-Fyy" } else { "-Fy" })
    .spawn()?;
  let st = child.wait()?;
  if !st.success() {
    return Err(eyre::eyre!("pacman exits with error: {st}"));
  }

  update_db(force)?;
  Ok(())
}

pub fn update_db(force: bool) -> Result<()> {
  for entry in std::fs::read_dir("/var/lib/pacman/sync")? {
    let entry = entry?;
    let path = entry.path();

    if path.extension() != Some(OsStr::new("files")) {
      continue;
    }

    process_repo(path, force)?;
  }

  Ok(())
}

fn process_repo(path: PathBuf, force: bool) -> Result<()> {
  let repo_name = path.file_stem().expect("unexpected .files filename")
    .to_str().expect("non-utf-8 .files filename?");
  print!("plocate-build {}.pacfiles ...", repo_name);
  let _ = std::io::stdout().flush();

  let target_path = path.with_extension("pacfiles");

  if !force {
    match target_path.metadata() {
      Ok(target_stat) => {
        let files_mtime = path.metadata()?.modified()?;
        if target_stat.modified()? > files_mtime {
          println!(" fresh.");
          return Ok(());
        }
      }
      Err(e) if e.kind() == ErrorKind::NotFound => { },
      Err(e) => { return Err(e.into()); }
    }
  }

  let f = File::open(&path)?;
  let f = BufReader::new(f);
  let ai = compress_tools::ArchiveIterator::from_read(f)?;

  let tmpfile = tempfile::NamedTempFile::new()?;

  {
    let mut f = BufWriter::new(tmpfile.as_file());
    let mut buffer = Vec::new();
    let mut pkg = String::new();
    let mut is_files = false;

    for a in ai {
      match a {
        ArchiveContents::StartOfEntry(name, _) => {
          if !name.ends_with("/files") {
            is_files = false;
            continue;
          }
          pkg += name.split_once('/').unwrap().0;
          is_files = true;
        },

        ArchiveContents::EndOfEntry => {
          match std::str::from_utf8(&buffer) {
            Ok(s) => {
              output_files(&mut f, &pkg, s)?;
            }
            Err(e) => {
              error!(%pkg, "files content is not utf-8 encoded: {:?}", e);
            }
          }
          buffer.clear();
          pkg.clear();
        },

        ArchiveContents::DataChunk(data) => {
          if is_files {
            buffer.extend_from_slice(&data);
          }
        },

        ArchiveContents::Err(e) => {
          return Err(e.into());
        },
      }
    }
  }

  info!("calling plocate-build");
  let tmp_path = tmpfile.into_temp_path();
  let mut child = Command::new("plocate-build")
    .args(["-p", "-l", "no",
      tmp_path.as_os_str().to_str().unwrap(),
      target_path.as_os_str().to_str().unwrap(),
    ])
    .spawn()?;
  let st = child.wait()?;
  if !st.success() {
    return Err(eyre::eyre!("plocate-build exits with error: {st}"));
  }

  let perm = fs::Permissions::from_mode(0o644);
  fs::set_permissions(target_path, perm)?;
  println!(" done.");

  Ok(())
}

fn output_files<W: Write>(
  mut f: W,
  pkg: &str,
  contents: &str,
) -> Result<()> {
  for line in contents.lines().skip(1) {
    writeln!(f, "{pkg}/{line}")?;
  }

  Ok(())
}

use std::path::Path;
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::mem::swap;

use tracing::debug;
use eyre::Result;
use rusqlite::Connection;

fn convert_files_one(files: &Path, repo: &str, db: &mut Connection) -> Result<()> {
  let it = FilesReader::new(files)?;

  db.execute("DELETE FROM files WHERE repo = ?1", [repo])?;

  let mut stmt = db.prepare(r"
  INSERT INTO files
  (repo, pkgname, path)
  VALUES (?1, ?2, ?3)
  ")?;

  for item in it {
    let (pkgname, filelist) = item?;
    for file in filelist.split('\n').skip(1) {
      if file == "" { // eof
        break;
      }
      stmt.execute(&[repo, &pkgname, file])?;
    }
  }

  Ok(())
}

fn may_convert_files_one(files: &Path, db: &mut Connection) -> Result<()> {
  let file_mtime = files.metadata()?.modified()?;
  let file_mtime = file_mtime.duration_since(std::time::UNIX_EPOCH)?.as_secs();
  let repo = files.file_stem().unwrap().to_str().unwrap();

  let last_mtime: Option<u64> = {
    let mut stmt = db.prepare("SELECT mtime FROM repoinfo WHERE repo = ?1")?;
    let x = stmt.query([repo])?.next()?.map(|x| x.get_unwrap(0));
    x
  };

  if last_mtime.is_some() && file_mtime <= last_mtime.unwrap() {
    debug!(repo=?repo, "fresh");
    Ok(())
  } else {
    debug!(repo=?repo, "converting");
    // prepare & transaction don't work together
    // https://github.com/rusqlite/rusqlite/issues/508
    db.execute_batch("BEGIN")?;
    let r = convert_files_one(files, repo, db);
    if r.is_err() {
      db.execute_batch("ROLLBACK")?;
    } else {
      if last_mtime.is_some() {
        db.execute("UPDATE repoinfo SET mtime = ?1 WHERE repo = ?2", (file_mtime, repo))?;
      } else {
        db.execute("INSERT INTO repoinfo (mtime, repo) VALUES (?1, ?2)",
        (file_mtime, repo))?;
      }
      db.execute_batch("COMMIT")?;
    }
    r
  }
}

pub fn may_convert_files<P: AsRef<Path>>(dir: P, db: &mut Connection) -> Result<()> {
  for entry in std::fs::read_dir(dir)? {
    let entry = entry?;
    let path = entry.path();
    if path.extension() == Some(OsStr::new("files")) {
      may_convert_files_one(&path, db)?;
    }
  }

  Ok(())
}

struct FilesReader {
  ai: compress_tools::ArchiveIterator<BufReader<File>>,
  buffer: Vec<u8>,
  is_desc: bool,
  is_files: bool,
  pkgname: String,
  name: String,
}

impl FilesReader {
  fn new(files: &Path) -> Result<Self> {
    let f = File::open(files)?;
    let f = BufReader::new(f);
    Ok(Self {
      ai: compress_tools::ArchiveIterator::from_read(f)?,
      buffer: Vec::new(),
      is_desc: false,
      is_files: false,
      pkgname: String::new(),
      name: String::new(),
    })
  }

  fn get_package_name(&mut self) -> Result<()> {
    let mut data = Vec::new();
    swap(&mut self.buffer, &mut data);
    let s = match String::from_utf8(data) {
      Ok(s) => s,
      Err(e) => return Err(
        eyre::eyre!("{} is not utf-8 encoded: {:?}", self.name, e)
      ),
    };

    let name = s.split('\n')
      .skip_while(|line| *line != "%NAME%")
      .nth(1);

    match name {
      None => Err(eyre::eyre!("{} is malformed", self.name)),
      Some(name) => {
        self.pkgname = String::from(name);
        Ok(())
      },
    }

  }
}

impl Iterator for FilesReader {
  type Item = Result<(String, String)>;

  fn next(&mut self) -> Option<Self::Item> {
    use compress_tools::ArchiveContents;

    while let Some(a) = self.ai.next() {
      match a {
        ArchiveContents::StartOfEntry(name, _) => {
          if name.ends_with("/desc") {
            self.is_desc = true;
          } else if name.ends_with("/files") {
            self.is_files = true;
          }
          self.buffer = Vec::new();
          self.name = name;
        },

        ArchiveContents::EndOfEntry => {
          if self.is_desc {
            if let Err(e) = self.get_package_name() {
              return Some(Err(e));
            }
            self.is_desc = false;
          } else if self.is_files {
            self.is_files = false;
            let mut ret = Vec::new();
            swap(&mut self.buffer, &mut ret);

            let mut pkgname = String::new();
            swap(&mut self.pkgname, &mut pkgname);

            return match String::from_utf8(ret) {
              Ok(s) => Some(Ok((pkgname, s))),
              Err(e) => Some(Err(
                eyre::eyre!("{}'s files are not utf-8 encoded: {:?}", self.pkgname, e)
              )),
            };
          }
        },

        ArchiveContents::DataChunk(data) => {
          if self.is_desc || self.is_files {
            self.buffer.extend_from_slice(&data);
          }
        },

        ArchiveContents::Err(e) => {
          return Some(Err(e.into()));
        },
      }
    }
    None
  }
}

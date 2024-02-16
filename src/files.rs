use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use std::mem::swap;

use eyre::Result;

pub struct FilesReader {
  ai: compress_tools::ArchiveIterator<BufReader<File>>,
  buffer: Vec<u8>,
  is_desc: bool,
  is_files: bool,
  pkgname: String,
  name: String,
}

impl FilesReader {
  pub fn new(files: &Path) -> Result<Self> {
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

pub struct FilesIter<'a> {
  files: std::str::Split<'a, char>,
}

impl<'a> FilesIter<'a> {
  pub fn new(s: &'a str) -> Self {
    let mut sp = s.split('\n');
    let _ = sp.next();
    Self {
      files: sp,
    }
  }
}

impl<'a> Iterator for FilesIter<'a> {
  type Item = &'a str;

  fn next(&mut self) -> Option<Self::Item> {
    match self.files.next() {
      Some("") => None,
      Some(f) => Some(f),
      None => None,
    }
  }
}

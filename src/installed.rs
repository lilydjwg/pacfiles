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

use std::collections::HashMap;
use std::io::Result as IoResult;

pub struct InstalledPackages {
  packages: HashMap<String, String>,
}

impl InstalledPackages {
  pub fn new() -> IoResult<Self> {
    let mut packages = HashMap::new();
    for entry in std::fs::read_dir("/var/lib/pacman/local")? {
      let entry = entry?;
      let path = entry.path();

      if !path.is_dir() {
        continue;
      }

      let name = path.file_name().unwrap().to_str().unwrap();

      let mut it = name.rsplitn(3, '-');
      it.next().unwrap(); // pkgrel
      it.next().unwrap(); // pkgver
      let pkgname = it.next().unwrap();
      let version = &name[pkgname.len()+1..];
      packages.insert(String::from(pkgname), String::from(version));
    }
    Ok(Self { packages })
  }

  pub fn package_version(&self, pkgname: &str) -> Option<&str> {
    self.packages.get(pkgname).map(|x| x.as_str())
  }
}

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

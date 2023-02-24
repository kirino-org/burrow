#![no_std]

extern crate alloc;
extern crate std;

use alloc::{boxed::Box, str::FromStr, string::String, vec::Vec};
use std::error::Error;
use std::path::PathBuf;

use nix::{
  dir::{Dir, Type},
  fcntl::OFlag,
  sys::stat::Mode,
  unistd::{chdir, getcwd},
};

#[derive(Clone)]
pub enum Burrow {
  File(String),
  Dir(String, BurrowMap),
}
pub type BurrowMap = Vec<Burrow>;
impl Burrow {
  pub fn name(&self) -> String {
    use Burrow::*;
    String::from(match self {
      File(n) => n,
      Dir(n, _) => n,
    })
  }
}

impl Burrow {
  pub fn index(path: &str, full_paths: bool) -> Result<Self, Box<dyn Error>> {
    let mut ret = BurrowMap::new();
    
    #[cfg_attr(feature = "tiny", allow(unused_mut))]
    let mut path = PathBuf::from_str(path)?;

    // Save current working directory
    let pre = getcwd()?;

    #[cfg(not(feature = "tiny"))]
    if full_paths {
      path = pre.join(path);
    }

    // Change dir to path so we can burrow
    chdir(&path)?;

    // Open dir
    if let Ok(mut dir) = Dir::open(".", OFlag::O_RDONLY, Mode::all()) {
      // Iterate over dir entries
      dir.iter().for_each(|x| {
        if let Ok(x) = x {
          let name = x.file_name().to_str().unwrap();
          if name != ".." && name != "." {
            ret.push(match x.file_type().unwrap() {
              Type::Directory => {
                Burrow::index(name, full_paths).unwrap_or(Burrow::File(String::from(
                  #[cfg(feature = "tiny")]
                  name,
                  #[cfg(not(feature = "tiny"))]
                  path.join(name).to_str().unwrap(),
                )))
              }
              _ => Burrow::File(String::from(
                #[cfg(feature = "tiny")]
                name,
                #[cfg(not(feature = "tiny"))]
                path.join(name).to_str().unwrap(),
              )),
            });
          }
        }
      });
    }

    chdir(&pre).unwrap();

    Ok(Burrow::Dir(String::from(
        path.to_str().unwrap(),
    ), ret))
  }
  pub fn index_many(
    paths: &[&str],
    full_paths: bool,
  ) -> Result<impl Iterator<Item = Self>, Box<dyn Error>> {
    let mut ret = BurrowMap::new();

    for p in paths {
      ret.push(Burrow::index(p, full_paths)?);
    }
    Ok(ret.into_iter())
  }
}

use std::collections::HashSet;
use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf};

use log::info;

pub struct Workspace {
    ingore: HashSet<PathBuf>,
    path: PathBuf,
}

impl Workspace {
    pub fn new(path: PathBuf) -> Workspace {
        let mut ingore = HashSet::new();
        ingore.insert(PathBuf::from(r"."));
        ingore.insert(PathBuf::from(r".."));
        ingore.insert(PathBuf::from(r".git"));
        ingore.insert(PathBuf::from(r"target"));

        Workspace { ingore, path }
    }

    fn read_files(&self, dir: &Path, cb: &mut FnMut(&DirEntry)) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if self
                    .ingore
                    .contains(&path.strip_prefix(self.path.clone()).unwrap().to_path_buf())
                {
                    info!("Ignoring path: {:?}", path);
                    continue;
                }

                if !self
                    .ingore
                    .contains(&path.strip_prefix(self.path.clone()).unwrap().to_path_buf())
                    && path.is_dir()
                {
                    self.read_files(&path, cb)?;
                } else {
                    cb(&entry);
                }
            }
        }
        Ok(())
    }

    pub fn list_files(&self) -> std::io::Result<Vec<PathBuf>> {
        let mut output = vec![];

        self.read_files(&self.path, &mut |entry| {
            output.push(entry.path());
        })?;

        Ok(output)
    }

    pub fn read_file(&self, path: PathBuf) -> std::io::Result<Vec<u8>> {
        fs::read(path)
    }
}

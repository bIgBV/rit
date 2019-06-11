use std::error::Error as StdError;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::prelude::*;
use std::path::PathBuf;

use crypto::{digest::Digest, sha1::Sha1};
use flate2::{write::ZlibEncoder, Compression};
use log::error;
use rand::prelude::*;

#[derive(Copy, Clone)]
pub enum ObjectKind {
    Blob,
    Tree,
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ObjectKind::Blob => write!(f, "{}", "blob"),
            ObjectKind::Tree => write!(f, "{}", "tree"),
        }
    }
}

#[derive(Debug)]
pub enum DbError {
    SerializeError(Error),
    IoError(std::io::Error),
}

impl StdError for DbError {
    fn description(&self) -> &str {
        format!("{}", *self).as_str()
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DbError::SerializeError(e) => write!(f, "Serialization error: {}", e.description()),
            DbError::IoError(io) => write!(f, "Io error: {}", io.description()),
        }
    }
}

pub struct Database {
    path: PathBuf,
}

type Error = Box<dyn std::error::Error>;
pub type SerializeResult<T> = Result<T, Error>;

pub trait Store<E>
where
    E: Into<Error>,
{
    fn otype(&self) -> ObjectKind;
    fn serialize(&self) -> SerializeResult<Vec<u8>>;
}

impl Database {
    pub fn new(path: PathBuf) -> Database {
        Database { path }
    }

    pub fn store<T, E>(&self, object: T) -> Result<(), Error>
    where
        T: Store<E>,
        E: Into<Error>,
    {
        let otype = object.otype().clone();
        let data = object
            .serialize()
            .map_err(|e| Box::new(DbError::SerializeError(e)))?;

        let mut content = format!("{} {}\0", otype, data.len()).into_bytes();

        content.extend(data.iter());

        let mut hasher = Sha1::new();
        hasher.input(&content);

        let oid = hasher.result_str().clone();

        self.write_object(oid, content)
            .map_err(|e| Box::new(DbError::IoError(e)))
    }

    fn write_object(&self, oid: String, content: Vec<u8>) -> std::io::Result<()> {
        let object_path = self.path.join(&oid[0..2]);
        let object_path = object_path.join(&oid[2..]);
        let dirname = self.path.join(&oid[0..2]);
        let temp_path = dirname.join(self.generate_temp_name());

        let result = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(temp_path.clone());

        let mut file = None;

        if result.is_err() {
            let err = result.unwrap_err();
            if err.kind() == std::io::ErrorKind::NotFound {
                fs::create_dir_all(dirname)?;

                let temp = OpenOptions::new()
                    .write(true)
                    .read(true)
                    .create(true)
                    .open(temp_path.clone())?;

                file = Some(temp);
            } else {
                error!("Error opening tem file: {}", err);
                return Err(err);
            }
        } else {
            // Should never fail
            file = Some(result.unwrap());
        }

        let mut file = file.unwrap();

        let mut e = ZlibEncoder::new(Vec::new(), Compression::fast());

        e.write_all(&content)?;
        let compressed = e.finish()?;

        file.write(&compressed)?;
        file.sync_all()?;

        fs::rename(temp_path, object_path)?;
        Ok(())
    }

    fn generate_temp_name(&self) -> String {
        let rand_name: [char; 6] = thread_rng().gen();
        let rand_name: String = rand_name[..].iter().collect();
        format!("tmp_obj_{:?}", rand_name)
    }
}

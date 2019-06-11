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

// #[derive(Debug, Display)]
// pub enum DbError {
//     #[display(fmt = "Serialization error: {}", e.description())]
//     SerializeError(Error),
//     #[display(fmt  =" error: {}", e.description())]
//     IoError(std::io::Error),
// }

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

    pub fn store<T, E>(&self, object: T) -> Result<(), DbError>
    where
        T: Store<E>,
        E: Into<Error>,
    {
        let otype = object.otype().clone();
        let data = object.serialize().map_err(|serializiation_err| DbError::SerializeError(serializiation_err.into()))?;

        let mut content = format!("{} {}\0", otype, data.len()).into_bytes();

        content.extend(data.iter());

        let mut hasher = Sha1::new();
        hasher.input(&content);

        // Not actualy a Result<String, Error>, just a String
        let oid = hasher.result_str();

        self.write_object(oid, &content)
            .map_err(|write_err| DbError::IoError(write_err))
    }

    fn write_object(&self, oid: &str, content: &[u8]) -> Result<(), Error> {
        let (object_path, dirname, temp_path) = destructure_oid(oid);

        if (path_exists(dirname).is_err()) {
            fs::create_dir_all(dirname)?;
        }

        match OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(temp_path.clone()) {
                Ok(file) => {
                    let mut e = ZlibEncoder::new(Vec::new(), Compression::fast());

                    e.write_all(content)?;
                    let compressed = e.finish()?;

                    file.write(&compressed)?;
                    file.sync_all()?;

                    fs::rename(temp_path, object_path)?;
                    Ok(())
                },
                Err(err) => {
                    error!("Error opening file: {}", err);

                    Err(err)
                }
            }
    }
}


fn generate_temp_name() -> String {
    let rand_name: [char; 6] = thread_rng().gen();
    let rand_name: String = rand_name[..].iter().collect();
    format!("tmp_obj_{:?}", rand_name)
}

fn destructure(oid: &str) -> () {
    let object_path = self.path.join(&oid[0..2]);
    let object_path = object_path.join(&oid[2..]);
    let dirname = self.path.join(&oid[0..2]);
    let temp_path = dirname.join(self.generate_temp_name());

    (object_path, dirname, temp_path)
}

fn checkPath(path: PathBuf) -> bool {
    if let Ok(_) = fs::metadata(path) {
        true
    }

    false
}
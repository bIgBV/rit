use crate::database::{ObjectKind, SerializeResult, Store};

use std::error::Error;
use std::fmt;

use derive_more::Display;

pub struct Blob {
    pub data: Vec<u8>,
    pub oid: String,
    otype: ObjectKind,
}

impl Blob {
    pub fn new(data: Vec<u8>) -> Blob {
        Blob {
            data,
            oid: String::new(),
            otype: ObjectKind::Blob,
        }
    }
}

#[derive(Debug, Display)]
pub enum BlobError {}

impl Error for BlobError {}

impl Store<BlobError> for Blob {
    fn otype(&self) -> ObjectKind {
        self.otype
    }

    fn serialize(&mut self) -> SerializeResult<Vec<u8>> {
        Ok(self.data.clone())
    }
}

use crate::database::{ObjectKind, SerializeResult, Store};
use crate::entry::Entry;

use std::error::Error;
use std::fmt;

use derive_more::Display;

const MODE: &str = "100644";

pub struct Tree<'a> {
    pub oid: String,
    entries: Vec<Entry<'a>>,
    otype: ObjectKind,
}

impl<'a> Tree<'a> {
    pub fn new(entries: Vec<Entry<'a>>) -> Tree<'a> {
        Tree {
            oid: String::new(),
            otype: ObjectKind::Tree,
            entries,
        }
    }
}

#[derive(Debug, Display)]
pub enum TreeError {}

impl Error for TreeError {}

impl<'a> Store<TreeError> for Tree<'a> {
    fn otype(&self) -> ObjectKind {
        self.otype
    }

    fn serialize(&mut self) -> SerializeResult<Vec<u8>> {
        self.entries[..].sort_by_key(|entry| entry.name);

        Ok(self
            .entries
            .iter()
            .map(|entry| format!("{:<7}{}\0{}", MODE, entry.name.display(), entry.oid))
            .collect::<String>()
            .into_bytes())
    }
}

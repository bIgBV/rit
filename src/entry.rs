use std::path::PathBuf;

pub struct Entry<'a> {
    pub name: &'a PathBuf,
    pub oid: String,
}

impl<'a> Entry<'a> {
    pub fn new(name: &PathBuf, oid: String) -> Entry {
        Entry { name, oid }
    }
}

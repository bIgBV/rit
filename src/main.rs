use std::env;
use std::fs;

use log::{error, info};
use pretty_env_logger;

mod blob;
mod database;
mod entry;
mod tree;
mod workspace;

use blob::Blob;
use database::Database;
use entry::Entry;
use tree::Tree;
use workspace::Workspace;

type Error = Box<dyn std::error::Error>;

fn run_app() -> Result<(), Error> {
    let args = env::args();

    match args.into_iter().skip(1).next().as_ref().map(|s| &s[..]) {
        Some("init") => {
            let cur_dir = env::current_dir()?;
            let git_dir = cur_dir.join(".git");

            for dir in &["objects", "refs"] {
                fs::create_dir_all(git_dir.join(dir))?;
            }

            println!("Initialized empty rit repository at {:?}", git_dir);
        }
        Some("commit") => {
            let cur_dir = env::current_dir()?;
            let git_dir = cur_dir.join(".git");
            let db_dir = git_dir.join("objects");

            let workspace = Workspace::new(cur_dir);
            let database = Database::new(db_dir);

            let entries = vec![];

            for path in workspace.list_files()?.iter() {
                info!("Handling path: {:?}", path);
                // TODO: Handle directories
                if path.is_dir() {
                    continue;
                }
                let data = workspace.read_file(path.to_path_buf())?;
                let blob = Blob::new(data);

                database.store(blob)?;
                entries.push(Entry::new(path, blob.oid));
            }

            let tree = Tree::new(entries);

            database.store(tree);

            info!("Tree: {}", tree.oid);
        }
        Some(val) => {
            eprintln!("rit: {} is not a rit command", val);
        }
        _ => eprintln!("Please pass a command"),
    }

    Ok(())
}

fn main() {
    pretty_env_logger::init();

    std::process::exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            error!("error: {}", err);
            1
        }
    });
}

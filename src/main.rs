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

pub type Error = Box<dyn std::error::Error>;
const APP_NAME: &'static str = env!("CARGO_PKG_NAME");

fn run_app() -> Result<(), Error> {
    let args = env::args();

    match args.into_iter().skip(1).next().as_ref().map(|s| &s[..]) {
        Some("init") => {
            let cur_dir = env::current_dir()?;
            let git_dir = cur_dir.join(".git");

            for dir in &["objects", "refs"] {
                fs::create_dir_all(git_dir.join(dir))?;
            }

            println!("Initialized empty {} repository at {:?}", APP_NAME, git_dir);
        }
        Some("commit") => {
            let cur_dir = env::current_dir()?;
            let git_dir = cur_dir.join(".git");
            let db_dir = git_dir.join("objects");

            let workspace = Workspace::new(cur_dir);
            let database = Database::new(db_dir);

            let paths = workspace.list_files()?;

            let blobs: Vec<Blob> = paths
                .iter()
                .map(|path| workspace.read_file(path.to_path_buf()))
                .filter_map(Result::ok)
                .map(Blob::new)
                .collect();

            let entries = paths
                .iter()
                .zip(blobs.into_iter())
                .map(|(path, blob)| {
                    let oid = blob.oid.clone();
                    database.store(blob).expect("Unable to serialize blob");

                    Entry::new(path, oid)
                })
                .collect();

            let tree = Tree::new(entries);

            info!("Tree: {}", tree.oid);
            database.store(tree)?;
        }
        Some(val) => {
            eprintln!(
                "{appname}: {} is not a {appname} command",
                val,
                appname = APP_NAME
            );
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

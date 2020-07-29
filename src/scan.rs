use failure::Error;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};
use log::{error, info};
use rayon::prelude::*;

// TODO: .img files should also be considered
#[inline]
fn is_tarball(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".tar.xz"))
        .unwrap_or(false)
}

pub fn collect_files<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .contents_first(true)
        .into_iter()
        .filter_entry(|e| is_tarball(e))
    {
        if let Ok(entry) = entry {
            files.push(entry.into_path());
        } else if let Err(e) = entry {
            error!("Could not stat() the entry: {}", e);
        }
    }

    Ok(files)
}

pub fn scan_files(files: &Vec<PathBuf>) {
    files.par_iter().for_each(|f| {
        println!("{:?}", f);
    });
}

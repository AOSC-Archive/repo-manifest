use crate::parser::{get_splitted_name, Tarball};
use failure::{format_err, Error};
use log::{error, info};
use parking_lot::Mutex;
use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::{
    convert::TryInto,
    fs::File,
    io::{Read, Seek, SeekFrom},
    sync::Arc,
};
use walkdir::{DirEntry, WalkDir};
use xz2::read::XzDecoder;

macro_rules! unwrap_or_show_error {
    ($m:tt, $p:expr, $f:stmt) => {{
        let tmp = { $f };
        if let Err(e) = tmp {
            error!($m, $p, e);
            return;
        }
        tmp.unwrap()
    }};
    ($m:tt, $p:expr, $x:ident) => {{
        if let Err(e) = $x {
            error!($m, $p, e);
            return;
        }
        $x.unwrap()
    }};
}

// TODO: .img files should also be considered
#[inline]
fn is_tarball(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.ends_with(".tar.xz"))
        .unwrap_or(false)
}

/// Calculate the Sha256 checksum of the given stream
pub fn sha256sum<R: Read>(mut reader: R) -> Result<String, Error> {
    let mut hasher = Sha256::new();
    std::io::copy(&mut reader, &mut hasher)?;

    Ok(hex::encode(hasher.finalize()))
}

/// Calculate the decompressed size of the given tarball
pub fn calculate_decompressed_size<R: Read>(reader: R) -> Result<u64, Error> {
    let mut buffer = [0u8; 4096];
    let mut decompress = XzDecoder::new(reader);
    loop {
        let size = decompress.read(&mut buffer)?;
        if size < 1 {
            break;
        }
    }

    Ok(decompress.total_out())
}

pub fn collect_files<P: AsRef<Path>>(root: P) -> Result<Vec<PathBuf>, Error> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root)
        .contents_first(true)
        .into_iter()
    {
        if let Ok(entry) = entry {
            if !is_tarball(&entry) {
                continue;
            }
            files.push(entry.into_path());
        } else if let Err(e) = entry {
            error!("Could not stat() the entry: {}", e);
        }
    }

    Ok(files)
}

pub fn scan_files(files: &Vec<PathBuf>, root_path: &str) -> Result<Vec<Tarball>, Error> {
    let results: Vec<Tarball> = Vec::new();
    let results_shared = Arc::new(Mutex::new(results));
    files.par_iter().for_each(|p| {
        info!("Scanning {}...", p.display());
        let rel_path = p.strip_prefix(root_path);
        let path = unwrap_or_show_error!(
            "Could get the relative path {}: {:?}",
            p.display(),
            rel_path
        );
        let filename = unwrap_or_show_error!(
            "Could not determine filename {}: {}",
            p.display(),
            path.file_name()
                .ok_or_else(|| format_err!("None value found"))
        );
        let names = unwrap_or_show_error!(
            "Could not parse the filename {}: {}",
            p.display(),
            get_splitted_name(&filename.to_string_lossy())
                .ok_or_else(|| format_err!("None value found"))
        );
        let mut f = unwrap_or_show_error!("Could not open {}: {}", p.display(), File::open(p));
        let real_size = unwrap_or_show_error!(
            "Could not read as xz stream {}: {}",
            p.display(),
            calculate_decompressed_size(&f)
        );
        let inst_size: i64 = real_size.try_into().unwrap();
        let pos = unwrap_or_show_error!(
            "Could not ftell() {}: {}",
            p.display(),
            f.seek(SeekFrom::Current(0))
        );
        let download_size: i64 = pos.try_into().unwrap();
        unwrap_or_show_error!(
            "Could not seek() {}: {}",
            p.display(),
            f.seek(SeekFrom::Start(0))
        );
        let sha256sum = unwrap_or_show_error!(
            "Could not update sha256sum of {}: {}",
            p.display(),
            sha256sum(&f)
        );
        let mut results = results_shared.lock();
        results.push(Tarball {
            arch: names.2,
            date: names.1,
            variant: names.0,
            download_size,
            inst_size,
            path: path.to_string_lossy().to_string(),
            sha256sum,
        });
    });

    Ok(Arc::try_unwrap(results_shared).unwrap().into_inner())
}

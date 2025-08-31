//! For simplicity, we will
//! * use only (regular) file or dir
//! * ignore r/w permissions

use std::{
    io::Write,
    path::{Path, PathBuf},
};

use base64::Engine;

pub enum EntryType {
    FILE,
    DIR,
}

#[allow(dead_code)]
pub fn ls(dir: impl AsRef<Path>, exclude_dir: bool) -> anyhow::Result<Vec<(PathBuf, EntryType)>> {
    let mut res = Vec::new();

    for entry in ::std::fs::read_dir(&dir)? {
        let entry = if let Ok(entry) = entry {
            entry
        } else {
            // ignore error
            continue;
        };
        let ftype = if let Ok(ftype) = entry.file_type() {
            ftype
        } else {
            // ignore error
            continue;
        };

        if ftype.is_dir() && !exclude_dir {
            res.push((entry.file_name().into(), EntryType::DIR));
        } else if ftype.is_file() {
            res.push((entry.file_name().into(), EntryType::FILE));
        }
    }

    Ok(res)
}

pub fn ls_recursive(
    dir: impl AsRef<Path>,
    exclude_dir: bool,
) -> anyhow::Result<Vec<(PathBuf, EntryType)>> {
    let mut res = Vec::new();
    ls_rec_body(&mut res, dir.as_ref(), "".as_ref(), exclude_dir)?;

    Ok(res)
}

fn ls_rec_body(
    res: &mut Vec<(PathBuf, EntryType)>,
    dir: &Path,
    relpath: &Path,
    exclude_dir: bool,
) -> anyhow::Result<()> {
    for entry in ::std::fs::read_dir(&dir)? {
        let entry = if let Ok(entry) = entry {
            entry
        } else {
            // ignore error
            continue;
        };
        let ftype = if let Ok(ftype) = entry.file_type() {
            ftype
        } else {
            // ignore error
            continue;
        };

        if ftype.is_dir() && !exclude_dir {
            let rel = relpath.join(entry.file_name());
            res.push((rel, EntryType::DIR));
        } else if ftype.is_file() {
            let rel = relpath.join(entry.file_name());
            res.push((rel, EntryType::FILE));
        }
        if ftype.is_dir() {
            let rel = relpath.join(entry.file_name());
            // ignore error
            let _ = ls_rec_body(res, &entry.path(), &rel, exclude_dir);
        }
    }

    Ok(())
}

fn compress_to_base64(src: &[u8]) -> String {
    let compressed = compress(src);

    base64::prelude::BASE64_STANDARD_NO_PAD.encode(compressed)
}

fn decompress_from_base64(src: &str) -> anyhow::Result<Vec<u8>> {
    let compressed = base64::prelude::BASE64_STANDARD_NO_PAD.decode(src)?;
    let decompressed = decompress(&compressed);

    Ok(decompressed)
}

fn compress(src: &[u8]) -> Vec<u8> {
    let mut en = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::best());
    en.write_all(src).unwrap();
    en.finish().unwrap()
}

fn decompress(src: &[u8]) -> Vec<u8> {
    let mut de = flate2::write::ZlibDecoder::new(Vec::new());
    de.write_all(src).unwrap();
    de.finish().unwrap()
}

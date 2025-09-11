//! For simplicity, we will
//! * use only (regular) file or dir
//! * ignore r/w permissions

use std::path::{Path, PathBuf};

use anyhow::bail;

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

type FsImage = Vec<FsImageEntry>;

#[derive(serde::Serialize, serde::Deserialize)]
struct FsImageEntry {
    #[serde(rename = "p")]
    path: String,
    #[serde(rename = "d")]
    data_base64: String,
}

pub fn create_fs_image(dir: impl AsRef<Path>) -> anyhow::Result<String> {
    let dir = dir.as_ref();
    let list = ls_recursive(dir, true)?;
    let mut obj = FsImage::new();

    for (path, _) in list {
        let fullpath = dir.join(&path);
        let data = ::std::fs::read(&fullpath)?;
        let data_base64 = compress_to_base64(&data);
        let entry = FsImageEntry {
            path: path.to_str().unwrap().to_string(),
            data_base64,
        };
        obj.push(entry);
    }

    Ok(serde_json::to_string(&obj)?)
}

pub fn import_fs_image(json: &str, dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let image: FsImage = serde_json::from_str(json)?;

    // move to "/"
    struct RestoreCurrentDir(PathBuf);
    impl Drop for RestoreCurrentDir {
        fn drop(&mut self) {
            if let Err(e) = std::env::set_current_dir(&self.0) {
                log::error!("{e:#}");
            }
        }
    }
    let cdir = std::env::current_dir()?;
    std::env::set_current_dir("/")?;
    let _restore = RestoreCurrentDir(cdir);

    // delete and create empty dir
    let dir = dir.as_ref();
    if ::std::fs::exists(dir)? {
        if dir.is_dir() {
            ::std::fs::remove_dir_all(dir)?;
        } else {
            ::std::fs::remove_file(dir)?;
        }
    }
    ::std::fs::create_dir_all(dir)?;

    for entry in image {
        let path = dir.join(entry.path);
        // forbid ".."
        if path
            .components()
            .any(|comp| comp == ::std::path::Component::ParentDir)
        {
            bail!("Invalid path");
        }
        let data = decompress_from_base64(&entry.data_base64)?;
        ::std::fs::write(&path, &data)?;
        log::debug!("Import {} ({} B)", path.to_str().unwrap(), data.len());
    }

    // restore working dir
    Ok(())
}

fn compress_to_base64(src: &[u8]) -> String {
    use base64::Engine;

    let compressed = compress(src);

    base64::prelude::BASE64_STANDARD_NO_PAD.encode(compressed)
}

fn decompress_from_base64(src: &str) -> anyhow::Result<Vec<u8>> {
    use base64::Engine;

    let compressed = base64::prelude::BASE64_STANDARD_NO_PAD.decode(src)?;
    let decompressed = decompress(&compressed)?;

    Ok(decompressed)
}

fn compress(src: &[u8]) -> Vec<u8> {
    use ::std::io::Write;

    let mut encoder = libflate::deflate::Encoder::new(Vec::new());
    encoder.write_all(src).unwrap();
    encoder.finish().into_result().unwrap()
}

fn decompress(src: &[u8]) -> anyhow::Result<Vec<u8>> {
    use ::std::io::Read;
    use anyhow::Context;

    let mut decoder = libflate::deflate::Decoder::new(src);
    let mut decoded_data = Vec::new();
    decoder
        .read_to_end(&mut decoded_data)
        .context("deflate error")?;

    Ok(decoded_data)
}

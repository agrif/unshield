use crate::format::{FileInfo, Format, FormatStep};

use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};

/// An interface for reading an InstallShield Z archive.
///
/// You can use this to read a Z archive out of any type that
/// implements [`Read`][Read] and [`Seek`][Seek].
///
///  [Read]: https://doc.rust-lang.org/std/io/trait.Read.html
///  [Seek]: https://doc.rust-lang.org/std/io/trait.Seek.html
#[derive(Debug)]
pub struct Archive<R> {
    inner: R,
    files: HashMap<String, FileInfo>,
}

impl<R> Archive<R>
where
    R: Read + Seek,
{
    /// Create a new Z archive from an underlying reader.
    ///
    /// This function will parse the file header and table of
    /// contents. If either of these fail, it will return an `Err`.
    pub fn new(mut inner: R) -> Result<Self> {
        let mut fmt = Format::new();
        loop {
            match fmt.next()? {
                FormatStep::Read(s, ref mut buf) => {
                    inner.seek(s)?;
                    inner.read_exact(buf)?;
                }

                FormatStep::Done(filesvec) => {
                    let mut files = HashMap::with_capacity(filesvec.len());
                    for f in filesvec.into_iter() {
                        files.insert(f.path.clone(), f);
                    }
                    return Ok(Archive { inner, files });
                }
            }
        }
    }

    /// List the files contained in the archive.
    pub fn list(&self) -> impl Iterator<Item = &FileInfo> {
        self.files.values()
    }

    fn find(&self, path: &str) -> Result<&FileInfo> {
        self.files
            .get(path)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "file not found"))
    }

    /// Load a file into a `Vec`.
    pub fn load(&mut self, path: &str) -> Result<Vec<u8>> {
        explode::explode(&self.load_compressed(path)?)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    /// Load a file into a `Vec` without decompressing it.
    pub fn load_compressed(&mut self, path: &str) -> Result<Vec<u8>> {
        let info = self.find(path)?;
        let size = info.size;
        let offset = info.offset;
        let mut ret = vec![0; size];
        self.inner.seek(SeekFrom::Start(offset))?;
        self.inner.read_exact(&mut ret)?;
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::Archive;
    use crate::examples::EXAMPLES;
    use std::io::Cursor;

    #[test]
    fn archive_new() {
        for (arcdata, _files) in EXAMPLES {
            let c = Cursor::new(arcdata);
            let _ar = Archive::new(c).unwrap();
        }
    }

    #[test]
    fn archive_list() {
        for (arcdata, files) in EXAMPLES {
            let c = Cursor::new(arcdata);
            let ar = Archive::new(c).unwrap();
            // are all files we can list expected?
            for file in ar.list() {
                let i = files.iter().find(|(name, _)| *name == file.path);
                if i.is_none() {
                    panic!("unexpected file {:?}", file.path);
                }
            }
        }
    }

    #[test]
    fn archive_load() {
        for (arcdata, files) in EXAMPLES {
            let c = Cursor::new(arcdata);
            let mut ar = Archive::new(c).unwrap();
            // do all expected files have the right contents?
            for (fname, contents) in files.iter() {
                let ours = ar.load(fname).unwrap();
                assert_eq!(ours, *contents);
            }
        }
    }
}

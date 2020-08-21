use crate::format::{FileInfo, Format, FormatStep};

use std::collections::HashMap;
use std::io::{Error, ErrorKind, Read, Result, Seek, SeekFrom};

#[derive(Debug)]
pub struct Archive<R> {
    inner: R,
    files: HashMap<String, FileInfo>,
}

impl<R> Archive<R>
where
    R: Read + Seek,
{
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

    pub fn list(&self) -> impl Iterator<Item = &FileInfo> {
        self.files.values()
    }

    fn find(&self, path: &str) -> Result<&FileInfo> {
        self.files
            .get(path)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "file not found"))
    }

    pub fn read_compressed(&mut self, path: &str) -> Result<Vec<u8>> {
        let info = self.find(path)?;
        let size = info.size;
        let offset = info.offset;
        let mut ret = vec![0; size];
        self.inner.seek(SeekFrom::Start(offset))?;
        self.inner.read_exact(&mut ret)?;
        Ok(ret)
    }

    pub fn read(&mut self, path: &str) -> Result<Vec<u8>> {
        explode::explode(&self.read_compressed(path)?)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }
}
use crate::format::{FileInfo, Format, FormatStep};

use futures_lite::io::{
    AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, Error, ErrorKind,
    Result, SeekFrom,
};
use std::collections::HashMap;

/// An interface for reading an InstallShield Z archive asynchronously.
///
/// You can use this to read a Z archive out of any type that
/// implements [`AsyncRead`][Read] and [`AsyncSeek`][Seek].
///
///  [Read]: https://docs.rs/futures-io/0.3/futures_io/trait.AsyncRead.html
///  [Seek]: https://docs.rs/futures-io/0.3/futures_io/trait.AsyncSeek.html
#[cfg_attr(docsrs, doc(cfg(feature = "async")))]
#[derive(Debug)]
pub struct AsyncArchive<R> {
    inner: R,
    files: HashMap<String, FileInfo>,
}

impl<R> AsyncArchive<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    /// Create a new Z archive from an underlying reader.
    ///
    /// This function will parse the file header and table of
    /// contents. If either of these fail, it will return an `Err`.
    pub async fn new(mut inner: R) -> Result<Self> {
        let mut fmt = Format::new();
        loop {
            match fmt.next()? {
                FormatStep::Read(s, ref mut buf) => {
                    inner.seek(s).await?;
                    inner.read_exact(buf).await?;
                }

                FormatStep::Done(filesvec) => {
                    let mut files = HashMap::with_capacity(filesvec.len());
                    for f in filesvec.into_iter() {
                        files.insert(f.path.clone(), f);
                    }
                    return Ok(AsyncArchive { inner, files });
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
    pub async fn load(&mut self, path: &str) -> Result<Vec<u8>> {
        explode::explode(&self.load_compressed(path).await?)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    /// Load a file into a `Vec` without decompressing it.
    pub async fn load_compressed(&mut self, path: &str) -> Result<Vec<u8>> {
        let info = self.find(path)?;
        let size = info.size;
        let offset = info.offset;
        let mut ret = vec![0; size];
        self.inner.seek(SeekFrom::Start(offset)).await?;
        self.inner.read_exact(&mut ret).await?;
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::AsyncArchive;
    use crate::examples::EXAMPLES;
    use smol::io::Cursor;

    #[test]
    fn archive_new() {
        for (arcdata, _files) in EXAMPLES {
            smol::block_on(async {
                let c = Cursor::new(arcdata);
                let _ar = AsyncArchive::new(c).await.unwrap();
            });
        }
    }

    #[test]
    fn archive_list() {
        for (arcdata, files) in EXAMPLES {
            smol::block_on(async {
                let c = Cursor::new(arcdata);
                let ar = AsyncArchive::new(c).await.unwrap();
                // are all files we can list expected?
                for file in ar.list() {
                    let i = files.iter().find(|(name, _)| *name == file.path);
                    if i.is_none() {
                        panic!("unexpected file {:?}", file.path);
                    }
                }
            });
        }
    }

    #[test]
    fn archive_load() {
        for (arcdata, files) in EXAMPLES {
            let c = Cursor::new(arcdata);
            smol::block_on(async {
                let mut ar = AsyncArchive::new(c).await.unwrap();
                // do all expected files have the right contents?
                for (fname, contents) in files.iter() {
                    let ours = ar.load(fname).await.unwrap();
                    assert_eq!(ours, *contents);
                }
            });
        }
    }
}

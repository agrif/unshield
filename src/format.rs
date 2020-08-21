use byteorder::{LittleEndian, ReadBytesExt};

use std::io::{Cursor, Error, ErrorKind, Read, Result, Seek, SeekFrom};

// all numbers are stored little-endian
type E = LittleEndian;

// file data begins here
const FILE_OFFSET: u64 = 255;

#[derive(Debug)]
pub struct Format {
    state: FormatState,
    buffer: Vec<u8>,
}

#[derive(Debug)]
pub enum FormatStep<'a> {
    Read(SeekFrom, &'a mut [u8]),
    Done(Vec<FileInfo>),
}

#[derive(Debug)]
enum FormatState {
    Start,
    ParseHeader,
    ParseToc(Header),
}

#[derive(Debug)]
struct Header {
    files: u16,
    size: u32,
    toc_offset: u32,
    dirs: u16,
}

/// Information about a file in an InstallShield Z archive.
///
/// You can get an iterator over these entries by using
/// [`Archive::list`](struct.Archive.html#method.list).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FileInfo {
    /// The name of the file (without directories).
    pub name: String,
    /// The full path of the file, using `\` to seperate directories.
    pub path: String,
    /// The compressed size of the file in the archive.
    pub size: usize,
    /// The offset within the archive where the file starts.
    pub offset: u64,
}

impl Format {
    pub fn new() -> Self {
        Format {
            state: FormatState::Start,
            buffer: vec![],
        }
    }

    fn read_string<R>(&self, mut input: R, len: usize) -> Result<String>
    where
        R: Read,
    {
        let mut buf = vec![0; len];
        input.read_exact(&mut buf)?;
        String::from_utf8(buf)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e))
    }

    pub fn next(&mut self) -> Result<FormatStep> {
        match self.state {
            FormatState::Start => {
                // read everything up until the files start
                self.state = FormatState::ParseHeader;
                self.buffer.resize(FILE_OFFSET as usize, 0);
                Ok(FormatStep::Read(SeekFrom::Start(0), &mut self.buffer))
            }

            FormatState::ParseHeader => {
                let mut c = Cursor::new(&self.buffer);
                let magic = c.read_u32::<E>()?;
                c.seek(SeekFrom::Current(8))?;
                let files = c.read_u16::<E>()?;
                c.seek(SeekFrom::Current(4))?;
                let size = c.read_u32::<E>()?;
                c.seek(SeekFrom::Current(19))?;
                let toc_offset = c.read_u32::<E>()?;
                c.seek(SeekFrom::Current(4))?;
                let dirs = c.read_u16::<E>()?;

                if magic != 0x8c655d13 {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "bad magic number",
                    ));
                }

                self.state = FormatState::ParseToc(Header {
                    files,
                    size,
                    toc_offset,
                    dirs,
                });
                self.buffer.resize((size - toc_offset) as usize, 0);
                Ok(FormatStep::Read(
                    SeekFrom::Start(toc_offset as u64),
                    &mut self.buffer,
                ))
            }

            FormatState::ParseToc(ref header) => {
                let mut c = Cursor::new(&self.buffer);
                let mut next_chunk = 0;

                let mut dirinfo = Vec::with_capacity(10);
                for _ in 0..header.dirs {
                    c.seek(SeekFrom::Start(next_chunk))?;
                    let dir_files = c.read_u16::<E>()?;
                    let chunk_size = c.read_u16::<E>()? as u64;
                    let dir_name_len = c.read_u16::<E>()? as usize;
                    let dir_name = self.read_string(&mut c, dir_name_len)?;

                    dirinfo.push((dir_files, dir_name));
                    next_chunk += chunk_size;
                }

                let mut offset = FILE_OFFSET;
                let mut fileinfo = Vec::with_capacity(10);
                for (dir_files, dir_name) in dirinfo {
                    for _ in 0..dir_files {
                        c.seek(SeekFrom::Start(next_chunk + 7))?;
                        let size = c.read_u32::<E>()? as usize;
                        c.seek(SeekFrom::Current(12))?;
                        let chunk_size = c.read_u16::<E>()? as u64;
                        c.seek(SeekFrom::Current(4))?;
                        let name_len = c.read_u8()? as usize;
                        let name = self.read_string(&mut c, name_len)?;
                        let path = if dir_name.len() > 0 {
                            dir_name.clone() + "\\" + &name
                        } else {
                            name.clone()
                        };

                        fileinfo.push(FileInfo {
                            offset,
                            size,
                            name,
                            path,
                        });
                        next_chunk += chunk_size;
                        offset += size as u64;
                    }
                }

                self.state = FormatState::Start;
                Ok(FormatStep::Done(fileinfo))
            }
        }
    }
}

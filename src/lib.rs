//! Extract files from InstallShield Z archives.
//!
//! This crate can open and extract files from [InstallShield Z
//! archives][z]. This archive format is used by version 3 of
//! InstallShield.
//!
//!  [z]: http://fileformats.archiveteam.org/wiki/InstallShield_Z
//!
//! # Examples
//!
//! Anything that implements [`Read`][Read] and [`Seek`][Seek] can be
//! read as an archive. Most commonly, this will be a [`File`][File].
//!
//!  [Read]: https://doc.rust-lang.org/std/io/trait.Read.html
//!  [Seek]: https://doc.rust-lang.org/std/io/trait.Seek.html
//!  [File]: https://doc.rust-lang.org/std/io/struct.File.html
//!
//! ```
//! # fn main() -> explode::Result<()> {
//! let mut some_file = std::fs::File::open("src/examples/demo.z")?;
//! let mut ar = unshield::Archive::new(some_file)?;
//!
//! let data = ar.load("subdir\\test.txt")?;
//!
//! for fileinfo in ar.list() {
//!     println!("{}", fileinfo.path);
//! }
//! # assert_eq!(data, b"fnord");
//! # Ok(()) }
//! ```

mod archive;
mod examples;
mod format;

pub use archive::Archive;
pub use format::FileInfo;

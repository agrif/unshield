# unshield

[![build status](https://api.travis-ci.com/agrif/unshield.svg?branch=master)](https://travis-ci.com/github/agrif/unshield)
[![crates.io](https://img.shields.io/crates/v/unshield.svg)](https://crates.io/crates/unshield)
[![docs.rs](https://docs.rs/unshield/badge.svg)](https://docs.rs/unshield)

Extract files from InstallShield Z archives.

This crate can open and extract files from [InstallShield Z
archives][z]. This archive format is used by version 3 of
InstallShield.

 [z]: http://fileformats.archiveteam.org/wiki/InstallShield_Z

## Command Line

This crate comes with a simple command line tool for extracting and
inspecting Z archives.

```bash
unshield list src/examples/demo.z
unshield extract src/examples/demo.z demo-out
```

## Examples

Anything that implements `Read` and `Seek` can be read as an
archive. Most commonly, this will be a `File`.

```rust
let mut some_file = std::fs::File::open("src/examples/demo.z")?;
let mut ar = unshield::Archive::new(some_file)?;

let data = ar.load("subdir\\test.txt")?;

for fileinfo in ar.list() {
    println!("{}", fileinfo.path);
}
```

## License

Licensed under the [MIT license](LICENSE). Unless stated otherwise,
any contributions to this work will also be licensed this way, with no
additional terms or conditions.

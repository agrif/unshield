#[cfg(test)]
pub const EXAMPLES: &[(&[u8], &[(&str, &[u8])])] = &[
    (
        include_bytes!("examples/demo.z"),
        &[
            ("hello.txt", b"Hello, world!"),
            ("subdir\\test.txt", b"fnord"),
        ],
    ),
    (
        include_bytes!("examples/undhr.z"),
        &[("undhr.md", include_bytes!("examples/undhr.md"))],
    ),
];

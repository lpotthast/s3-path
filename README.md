# s3-path

Construct safe S3 keys (informally named paths in this library).

This library only allows the following characters to be used in path components:

- `[a..z]`, `[A..Z]`, `[0..9]`, `-`, `_`, `.`

The following path components are explicitly forbidden to avoid any path traversal when the S3 key is used as a
filesystem path:

- `.`
- `..`

# Usage

```rust
fn owned() {
    let mut path = S3PathBuf::new();
    path.join("foo").unwrap();
    path.join("bar").unwrap();
    assert_that(path).has_display_value("foo/bar");

    let path = S3PathBuf::try_from(["foo/bar", "baz"]).unwrap();
    assert_that(path).has_display_value("foobar/baz");
    
    let path = S3PathBuf::try_from_str("foo/bar").unwrap();
    assert_that(path).has_display_value("foo/bar");
}

fn borrowed() {
    let path = S3Path::new([Cow::Borrowed("foo"), Cow::Borrowed("bar")]).unwrap();
    assert_that(path).has_display_value("foo/bar");
}
```

## TODO

- [ ] Check for maximum key limit? Should be 1024 bytes (of UTF-8) on Amazon S3. What about MinIO, Ceph, ...?

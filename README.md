# s3-path

Construct S3 keys (paths).

This library only allows the following characters to be used in components:

- `[a..z]`, `[A..Z]`, `[0..9]`, `-`, `_`, `.`

The following components are explicitly forbidden to avoid any path traversal when an S3 key is used as a filesystem
path:

- `.`
- `..`

# Usage

```rust
fn owned() {
    let mut path = S3PathBuf::new();
    path.join("foo").unwrap();
    path.join("bar").unwrap();
    assert_that(path).has_display_value("foo/bar");
}

fn borrowed() {
    let path = S3Path::try_from(["foo", "bar"]).unwrap();
    assert_that(path).has_display_value("foo/bar");
}
```

## TODO

- [ ] Check for maximum key limit? Should be 1024 bytes (of UTF-8) on Amazon S3. What about MinIO, Ceph, ...?

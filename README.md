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
use assertr::prelude::*;
use s3_path::{s3_path_buf, S3Path, S3PathBuf};

fn create_an_owned_path() {
    // Using the macro.
    let path = s3_path_buf!("foo", String::from("bar")).unwrap();
    assert_that(path).has_display_value("foo/bar");
    
    // From a list of components.
    let path = S3PathBuf::try_from(["foo", "bar"]).unwrap();
    assert_that(path).has_display_value("foo/bar");

    // From a single string (split at '/'s, only function performing a split).
    let path = S3PathBuf::try_from_str("foo/bar").unwrap();
    assert_that(path).has_display_value("foo/bar");

    // From manual calls to `join`.
    let mut path = S3PathBuf::new();
    path.join("foo").unwrap();
    path.join("bar").unwrap();
    assert_that(path).has_display_value("foo/bar");
}

fn create_a_borrowed_path() {
    // Using the macro. All args must be static.
    let path = s3_path!("foo", "bar").unwrap();
    assert_that(path).has_display_value("foo/bar");

    // From a list of components. All args must be static.
    let path = S3Path::new([Cow::Borrowed("foo"), Cow::Borrowed("bar")]).unwrap();
    assert_that(path).has_display_value("foo/bar");

    // From an owned path.
    let path = s3_path_buf!("foo", "bar").unwrap();
    let borrowed: &S3Path<'_> = path.as_path();
    assert_that(borrowed).has_display_value("foo/bar");
    assert_that(&path).is_equal_to(borrowed);
}
```

## Linting

```sh
cargo clippy -- -Dclippy::all -Dclippy::pedantic
```

## TODO

- [ ] Check for maximum key limit? Should be 1024 bytes (of UTF-8) on Amazon S3. What about MinIO, Ceph, ...?

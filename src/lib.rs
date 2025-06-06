pub mod error;
mod validation;

use crate::error::InvalidS3PathComponent;
use std::borrow::Cow;
use std::fmt::Formatter;
use std::ops::Deref;
use std::path::PathBuf;

/// Var-arg macro to create an `S3Path`, borrowing from the given string literals.
///
/// ```
/// use s3_path::s3_path;
///
/// let path = s3_path!("foo", "bar").unwrap();
/// ```
#[macro_export]
macro_rules! s3_path {
    ($($component:expr),* $(,)?) => {{
        let components = &[$(::std::borrow::Cow::Borrowed($component)),*];
        $crate::S3Path::new(components)
    }}
}

/// Var-arg macro to create an `S3Path` from individual components.
///
/// ```
/// use std::borrow::Cow;
/// use s3_path::s3_path_buf;
///
/// let path = s3_path_buf!("foo", String::from("bar"), Cow::Borrowed("baz")).unwrap();
/// ```
///
/// Every component passed into this macro must either
/// - be a `&'static str`
/// - an owned `String`
/// - or a `Cow<'static, str>`
///
/// str-slices of arbitrary lifetime cannot be used, as an `S3PathBuf` requires ownership,
/// and this API enforces that any allocation, if needed, is performed at call-site.
#[macro_export]
macro_rules! s3_path_buf {
    ($($component:expr),* $(,)?) => {{
        #[allow(unused_mut)] // In case zero components are passed in.
        let mut path = $crate::S3PathBuf::new();
        #[allow(unused_mut)] // In case zero components are passed in.
        let mut error = None;
        $(
            if error.is_none() {
                match path.push($component) {
                    Ok(_) => {},
                    Err(e) => error = Some(e),
                }
            }
        )*
        match error {
            Some(err) => Result::<$crate::S3PathBuf, $crate::error::InvalidS3PathComponent>::Err(err),
            None => Result::<$crate::S3PathBuf, $crate::error::InvalidS3PathComponent>::Ok(path),
        }
    }}
}

/// A borrowed, unsized S3 storage path.
///
// Must be repr(transparent) to safely convert from the slice.
#[repr(transparent)]
#[derive(PartialEq, Eq)]
pub struct S3Path<'i>([Cow<'i, str>]);

/// An owned S3 storage path.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct S3PathBuf {
    components: Vec<Cow<'static, str>>,
}

/// Allow comparisons between `S3Path` and `S3PathBuf`.
impl PartialEq<S3Path<'_>> for S3PathBuf {
    fn eq(&self, other: &S3Path<'_>) -> bool {
        self.as_path() == other
    }
}

/// Allow comparisons between `S3Path` and `S3PathBuf`.
impl<'i> PartialEq<&S3Path<'i>> for S3PathBuf {
    fn eq(&self, other: &&S3Path<'i>) -> bool {
        self.as_path() == *other
    }
}

/// Allow comparisons between `S3Path` and `S3PathBuf`.
impl<'i> PartialEq<&&S3Path<'i>> for S3PathBuf {
    fn eq(&self, other: &&&S3Path<'i>) -> bool {
        self.as_path() == **other
    }
}

/// Allow comparisons between `S3Path` and `S3PathBuf`.
impl PartialEq<S3PathBuf> for S3Path<'_> {
    fn eq(&self, other: &S3PathBuf) -> bool {
        self == other.as_path()
    }
}

/// Allow comparisons between `S3Path` and `S3PathBuf`.
impl PartialEq<S3PathBuf> for &S3Path<'_> {
    fn eq(&self, other: &S3PathBuf) -> bool {
        *self == other.as_path()
    }
}

/// Allow comparisons between `S3Path` and `S3PathBuf`.
impl PartialEq<S3PathBuf> for &&S3Path<'_> {
    fn eq(&self, other: &S3PathBuf) -> bool {
        **self == other.as_path()
    }
}

impl<'i> AsRef<S3Path<'i>> for S3Path<'i> {
    fn as_ref(&self) -> &S3Path<'i> {
        self
    }
}

impl<'i> AsRef<S3Path<'i>> for S3PathBuf {
    fn as_ref(&self) -> &S3Path<'i> {
        self
    }
}

impl AsRef<S3PathBuf> for S3PathBuf {
    fn as_ref(&self) -> &S3PathBuf {
        self
    }
}

fn write_components<C: AsRef<str>>(
    components: impl Iterator<Item = C>,
    f: &mut Formatter,
) -> Result<(), std::fmt::Error> {
    for (i, c) in components.enumerate() {
        if i > 0 {
            f.write_str("/")?;
        }
        f.write_str(c.as_ref())?;
    }
    Ok(())
}

impl std::fmt::Display for S3Path<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_components(self.0.iter(), f)?;
        Ok(())
    }
}

impl std::fmt::Debug for S3Path<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_components(self.0.iter(), f)?;
        Ok(())
    }
}

impl std::fmt::Display for S3PathBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_components(self.components.iter(), f)?;
        Ok(())
    }
}

impl std::fmt::Debug for S3PathBuf {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_components(self.components.iter(), f)?;
        Ok(())
    }
}

impl<'i> S3Path<'i> {
    /// Create a new `S3Path` from a slice of static `components`.
    ///
    /// # Errors
    ///
    /// Returns `Err` when any given `component`
    /// - is empty
    /// - contains characters other than: ascii alphanumeric characters, '-', '_' and '.'
    /// - is equal to `.` or `..`
    pub fn new(components: &'i [Cow<'i, str>]) -> Result<&'i S3Path<'i>, InvalidS3PathComponent> {
        for component in components {
            validation::validate_component(component)?;
        }
        // Safety: S3Path is repr(transparent) over [Cow<'i, str>].
        Ok(unsafe { &*(std::ptr::from_ref::<[Cow<'i, str>]>(components) as *const S3Path<'i>) })
    }

    /// Converts to an owned `S3PathBuf`.
    #[must_use]
    pub fn to_owned(&'i self) -> S3PathBuf {
        S3PathBuf {
            components: self.0.iter().map(|it| Cow::Owned(it.to_string())).collect(),
        }
    }

    /// Converts to an owned `S3PathBuf` and appends `component` to it after validating it.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the given `component`
    /// - is empty
    /// - contains characters other than: ascii alphanumeric characters, '-', '_' and '.'
    /// - is equal to `.` or `..`
    pub fn join<C: Into<Cow<'static, str>>>(
        &self,
        component: C,
    ) -> Result<S3PathBuf, InvalidS3PathComponent> {
        let mut path = self.to_owned();
        path.push(component)?;
        Ok(path)
    }

    /// Returns true if this path has no components.
    #[must_use]
    pub fn is_empty(&'i self) -> bool {
        self.0.is_empty()
    }

    /// Returns the number of components in this path.
    #[must_use]
    pub fn len(&'i self) -> usize {
        self.0.len()
    }

    /// Returns an iterator over the components of this path.
    pub fn components(&'i self) -> impl Iterator<Item = &'i str> {
        self.0.iter().map(std::convert::AsRef::as_ref)
    }

    /// Returns the component at the given index, or None if the index is out of bounds.
    pub fn get(&'i self, index: usize) -> Option<&'i str> {
        self.0.get(index).map(std::convert::AsRef::as_ref)
    }

    /// Returns the last component of this path, or None if the path is empty.
    pub fn last(&'i self) -> Option<&'i str> {
        self.0.last().map(std::convert::AsRef::as_ref)
    }

    /// Returns all but the last component of this path, or None if the path is empty.
    #[must_use]
    pub fn parent(&'i self) -> Option<&'i S3Path<'i>> {
        if self.0.is_empty() {
            None
        } else {
            let parent_slice = &self.0[..self.0.len() - 1];
            Some(
                // Safety: S3Path is repr(transparent) over [Cow<'i, str>]
                unsafe {
                    &*(std::ptr::from_ref::<[Cow<'i, str>]>(parent_slice) as *const S3Path<'i>)
                },
            )
        }
    }

    /// Convert this S3 path to a `std::path::PathBuf`, allowing you to use this S3 path as a
    /// system file path.
    ///
    /// Our strong guarantee that path components only consist of ascii alphanumeric characters,
    /// '-', '_' and '.' and that no path traversal components ('.' and '..') are allowed, makes
    /// this a safe operation.
    #[must_use]
    pub fn to_std_path_buf(&self) -> PathBuf {
        let mut path = PathBuf::new();
        for c in &self.0 {
            path.push(c.as_ref());
        }
        path
    }
}

// Deref - NameBuf can be automatically converted to &Name<'static>
impl Deref for S3PathBuf {
    type Target = S3Path<'static>;

    fn deref(&self) -> &Self::Target {
        // Safety: This is safe because Name is repr(transparent) over [Cow<'i, str>] and we
        // store [Cow<'static>, str>], with 'static outliving any lifetime 'i.
        unsafe {
            &*(std::ptr::from_ref::<[Cow<'static, str>]>(self.components.as_slice())
                as *const S3Path<'static>)
        }
    }
}

impl S3PathBuf {
    /// Creates an empty S3 path. Call `join` or `push` to extend it with additional path segments.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Validates and adds all `components` to the returned `S3PathBuf`.
    ///
    /// NO component is parsed for slashes ('/') to be split up further!
    ///
    /// # Errors
    ///
    /// Returns `Err` when any given component
    /// - is empty
    /// - contains characters other than: ascii alphanumeric characters, '-', '_' and '.'
    /// - is equal to `.` or `..`
    pub fn try_from<C: Into<Cow<'static, str>>, I: IntoIterator<Item = C>>(
        components: I,
    ) -> Result<Self, InvalidS3PathComponent> {
        let mut path = S3PathBuf::new();
        for component in components {
            path.push(component)?;
        }
        Ok(path)
    }

    /// Splits `string` at each occurrence of a `/`, then validates and add all components to the
    /// returned `S3PathBuf`.
    ///
    /// Multiple consecutive slashes, as in "foo//bar", are treated as one.
    ///
    /// # Errors
    ///
    /// Returns `Err` when any component read
    /// - contains characters other than: ascii alphanumeric characters, '-', '_' and '.'
    /// - is equal to `.` or `..`
    pub fn try_from_str(string: impl AsRef<str>) -> Result<Self, InvalidS3PathComponent> {
        let mut path = S3PathBuf::new();
        for c in string.as_ref().split('/') {
            // Skip empty components from consecutive slashes
            if !c.is_empty() {
                path.push(Cow::Owned(c.to_string()))?;
            }
        }
        Ok(path)
    }

    /// Adds `component` to the path after validating it.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the given component
    /// - is empty
    /// - contains characters other than: ascii alphanumeric characters, '-', '_' and '.'
    /// - is equal to `.` or `..`
    pub fn push(
        &mut self,
        component: impl Into<Cow<'static, str>>,
    ) -> Result<&mut Self, InvalidS3PathComponent> {
        let comp = component.into();
        validation::validate_component(&comp)?;
        self.components.push(comp);
        Ok(self)
    }

    /// Extend this path with a new segment in a new object.
    pub fn join(
        &self,
        component: impl Into<Cow<'static, str>>,
    ) -> Result<Self, InvalidS3PathComponent> {
        let mut clone = self.clone();
        clone.push(component)?;
        Ok(clone)
    }

    #[must_use]
    #[inline]
    pub fn as_path(&self) -> &S3Path<'_> {
        self
    }

    /// Pop the last component from this path, returning true if a component was removed
    pub fn pop(&mut self) -> Option<Cow<'static, str>> {
        self.components.pop()
    }
}

#[cfg(test)]
impl assertr::assertions::HasLength for S3PathBuf {
    fn length(&self) -> usize {
        self.len()
    }
}

#[cfg(test)]
mod test {
    use crate::S3PathBuf;
    use assertr::prelude::*;

    mod s3_path_buf {
        use crate::S3PathBuf;
        use assertr::prelude::*;

        #[test]
        fn new_is_initially_empty() {
            let path = S3PathBuf::new();
            assert_that(path).has_display_value("");
        }

        #[test]
        fn construct_using_new_and_push_components() {
            let mut path = S3PathBuf::new();
            path.push("foo").unwrap();
            path.push("bar").unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn try_from_str_parses_empty_string_as_empty_path() {
            let path = S3PathBuf::try_from_str("").unwrap();
            assert_that(path).has_display_value("");
        }

        #[test]
        fn try_from_str_parses_single_slash_as_empty_path() {
            let path = S3PathBuf::try_from_str("/").unwrap();
            assert_that(path).has_display_value("");
        }

        #[test]
        fn try_from_str_removes_leading_and_trailing_slashes() {
            let path = S3PathBuf::try_from_str("/foo/bar/").unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn try_from_str_ignores_repeated_slashes() {
            let path = S3PathBuf::try_from_str("foo/////bar").unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn construct_using_try_from_given_str() {
            let path = S3PathBuf::try_from_str("foo/bar").unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn construct_using_try_from_given_string() {
            let path = S3PathBuf::try_from_str(String::from("foo/bar")).unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn try_from_static_str_array() {
            let path = S3PathBuf::try_from(["foo", "bar"]).unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn try_from_string_array() {
            let path = S3PathBuf::try_from(["foo".to_string(), "bar".to_string()]).unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn reject_invalid_characters() {
            let mut path = S3PathBuf::new();
            let result = path.push("invalid/path");
            assert_that(result).is_err();

            let result = S3PathBuf::try_from_str("foo/bar$baz");
            assert_that(result).is_err();
        }

        #[test]
        fn push_mutates_original() {
            let mut foo = S3PathBuf::try_from_str("foo").unwrap();
            let foo_bar = foo.push("bar").unwrap();

            assert_that(foo_bar).has_display_value("foo/bar");
            assert_that(foo).has_display_value("foo/bar");
        }

        #[test]
        fn join_creates_clone() {
            let foo = S3PathBuf::try_from_str("foo").unwrap();
            let foo_bar = foo.join("bar").unwrap();

            assert_that(foo).has_display_value("foo");
            assert_that(foo_bar).has_display_value("foo/bar");
        }

        #[test]
        fn as_path() {
            let path_buf = S3PathBuf::try_from(["foo", "bar"]).unwrap();
            let path = path_buf.as_path();
            assert_that(path).has_display_value("foo/bar");
            let cloned = path.to_owned();
            drop(path_buf);
            assert_that(cloned).has_display_value("foo/bar");
        }

        #[test] // Function `is_empty` inherited through deref to S3Path!
        fn is_empty_returns_true_when_path_has_no_components() {
            let path_buf = S3PathBuf::new();
            assert_that(path_buf.is_empty()).is_true();
        }

        #[test] // Function `is_empty` inherited through deref to S3Path!
        fn is_empty_returns_false_when_path_has_components() {
            let path_buf = S3PathBuf::try_from(["foo", "bar"]).unwrap();
            assert_that(path_buf.is_empty()).is_false();
        }

        #[test] // Function `len` inherited through deref to S3Path!
        fn len_returns_number_of_components() {
            let path_buf = S3PathBuf::try_from(["foo", "bar"]).unwrap();
            assert_that(path_buf.len()).is_equal_to(2);
        }

        #[test] // Function `components` inherited through deref to S3Path!
        fn components_iterates_over_all_components() {
            let path_buf = S3PathBuf::try_from(["foo", "bar"]).unwrap();
            assert_that(path_buf.components()).contains_exactly(["foo", "bar"]);
        }

        #[test] // Function `get` inherited through deref to S3Path!
        fn get_returns_component_at_index() {
            let path_buf = S3PathBuf::try_from(["foo", "bar"]).unwrap();
            assert_that(path_buf.get(1)).is_some().is_equal_to("bar");
        }

        #[test] // Function `last` inherited through deref to S3Path!
        fn last_returns_last_component() {
            let path_buf = S3PathBuf::try_from(["foo", "bar"]).unwrap();
            assert_that(path_buf.last()).is_some().is_equal_to("bar");
        }

        #[test] // Function `parent` inherited through deref to S3Path!
        fn parent_returns_none_when_path_has_no_components() {
            let path_buf = S3PathBuf::new();
            assert_that(path_buf.parent()).is_none();
        }

        #[test] // Function `parent` inherited through deref to S3Path!
        fn parent_returns_empty_component_when_path_has_only_one_component() {
            let path_buf = S3PathBuf::try_from(["foo"]).unwrap();
            assert_that(path_buf.parent())
                .is_some()
                .has_display_value("");
        }

        #[test] // Function `parent` inherited through deref to S3Path!
        fn parent_returns_view_of_parent_path_when_path_has_multiple_components() {
            let path_buf = S3PathBuf::try_from(["foo", "bar", "baz"]).unwrap();
            assert_that(path_buf.parent())
                .is_some()
                .has_display_value("foo/bar");
        }

        #[test] // Function `to_std_path_buf` inherited through deref to S3Path!
        fn to_std_path_buf_returns_empty_path_buf_when_s3_path_has_zero_components() {
            let path_buf = S3PathBuf::new();
            assert_that(path_buf.to_std_path_buf().display()).has_display_value("");
        }

        #[test] // Function `to_std_path_buf` inherited through deref to S3Path!
        fn to_std_path_buf_joins_components_with_path_separator_does_not_add_slashes() {
            let path_buf = S3PathBuf::try_from(["foo", "bar"]).unwrap();
            assert_that(path_buf.to_std_path_buf().display()).has_display_value("foo/bar");
        }

        mod s3_path_buf_macro {
            use assertr::prelude::*;
            use std::borrow::Cow;

            #[test]
            fn taking_nothing() {
                let path = s3_path_buf!().unwrap();
                assert_that(path).has_display_value("");
            }

            #[test]
            fn taking_single_static_str() {
                let path = s3_path_buf!("foo").unwrap();
                assert_that(path).has_display_value("foo");
            }

            #[test]
            fn taking_owned_string_and_static_str() {
                let foo = String::from("foo");
                let path = s3_path_buf!(foo, "bar").unwrap();
                assert_that(path).has_display_value("foo/bar");
            }

            #[test]
            fn taking_owned_string_and_static_str_and_cow() {
                let foo = String::from("foo");
                let path = s3_path_buf!(foo, "bar", Cow::Borrowed("baz")).unwrap();
                assert_that(path).has_display_value("foo/bar/baz");
            }

            #[test]
            fn allows_a_trailing_comma() {
                let path = s3_path_buf!("foo",).unwrap();
                assert_that(path).has_display_value("foo");
            }
        }
    }

    mod s3_path {
        use crate::S3Path;
        use assertr::prelude::*;
        use std::borrow::Cow;

        #[test]
        fn new_is_initially_empty() {
            let path = S3Path::new(&[]).unwrap();
            assert_that(path).has_display_value("");
        }

        #[test]
        fn construct_using_new() {
            let path = S3Path::new(&[Cow::Borrowed("foo"), Cow::Borrowed("bar")]).unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn to_owned_converts_to_owned_path() {
            let path = s3_path!("foo", "bar").unwrap();
            let path_owned = path.to_owned();
            assert_that(path_owned).has_display_value("foo/bar");
        }

        #[test]
        fn join_converts_to_owned_path_and_appends_component() {
            let path = s3_path!("foo", "bar").unwrap();
            let path_owned = path.join("baz").unwrap();
            assert_that(path_owned).has_display_value("foo/bar/baz");
        }

        mod s3_path_macro {
            use assertr::prelude::*;

            #[test]
            fn handles_zero_components() {
                let path = s3_path!().unwrap();
                assert_that(path).has_display_value("");
            }

            #[test]
            fn handles_one_component() {
                let path = s3_path!("foo").unwrap();
                assert_that(path).has_display_value("foo");
            }

            #[test]
            fn handles_multiple_components() {
                let path = s3_path!("foo", "bar").unwrap();
                assert_that(path).has_display_value("foo/bar");
            }

            #[test]
            fn allows_a_trailing_comma() {
                let path = s3_path!("foo",).unwrap();
                assert_that(path).has_display_value("foo");
            }
        }
    }

    mod validation {
        use crate::S3PathBuf;
        use assertr::prelude::*;

        #[test]
        fn reject_invalid_characters() {
            assert_that(S3PathBuf::try_from(["foo-bar"]))
                .is_ok()
                .has_display_value("foo-bar");
            assert_that(S3PathBuf::try_from(["foo_bar"]))
                .is_ok()
                .has_display_value("foo_bar");
            assert_that(S3PathBuf::try_from(["foo.bar"]))
                .is_ok()
                .has_display_value("foo.bar");
            assert_that(S3PathBuf::try_from([".test"]))
                .is_ok()
                .has_display_value(".test");
            assert_that(S3PathBuf::try_from(["..test"]))
                .is_ok()
                .has_display_value("..test");

            assert_that(S3PathBuf::try_from(["foo:bar"])).is_err();
            assert_that(S3PathBuf::try_from(["foo;bar"])).is_err();
            assert_that(S3PathBuf::try_from(["foo$bar"])).is_err();
            assert_that(S3PathBuf::try_from(["foo&bar"])).is_err();
            assert_that(S3PathBuf::try_from(["foo#bar"])).is_err();
            assert_that(S3PathBuf::try_from(["foo/bar"])).is_err();
            assert_that(S3PathBuf::try_from(["foo|bar"])).is_err();
            assert_that(S3PathBuf::try_from(["foo\\bar"])).is_err();
            assert_that(S3PathBuf::try_from(["."])).is_err();
            assert_that(S3PathBuf::try_from([".."])).is_err();
        }
    }

    mod take_any_path {
        use crate::{S3Path, S3PathBuf};

        fn take_any_path<'p>(path: impl AsRef<S3Path<'p>>) {
            let _path: &S3Path<'p> = path.as_ref();
        }

        #[test]
        fn takes_owned_s3_path_buf() {
            take_any_path(S3PathBuf::new());
        }

        #[test]
        fn takes_borrowed_s3_path_buf() {
            take_any_path(&S3PathBuf::new());
        }

        #[test]
        fn takes_s3_path() {
            take_any_path(S3Path::new(&[]).unwrap());
        }
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn comparison_between_types() {
        let path_buf = S3PathBuf::try_from(["foo", "bar"]).unwrap();
        let path = path_buf.as_path();

        assert_that(path == path).is_true();
        assert_that(path_buf == path_buf).is_true();

        assert_that(path == path_buf).is_true();
        assert_that(path == &path_buf).is_true();
        assert_that(&path == path_buf).is_true();
        assert_that(&path == &path_buf).is_true();

        assert_that(path_buf == path).is_true();
        assert_that(path_buf == &path).is_true();
        assert_that(&path_buf == path).is_true();
        assert_that(&path_buf == &path).is_true();

        // Also works with assertr.
        assert_that(&path_buf).is_equal_to(path);
        assert_that(s3_path_buf!("foo", "bar"))
            .is_ok()
            .is_equal_to(path);
    }
}

pub mod error;
mod validation;

use crate::error::InvalidS3PathComponent;
use std::borrow::Cow;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::path::PathBuf;

pub type BorrowedS3PathComp<'i> = Cow<'i, str>;
pub type OwnedS3PathComp = Cow<'static, str>;

pub trait S3PathIter<C>: IntoIterator<Item = C> + Clone {}

impl<'i, C: Into<BorrowedS3PathComp<'i>>, II: IntoIterator<Item = C> + Clone> S3PathIter<C> for II {}

/// A borrowed S3 storage path.
#[derive(Clone)]
pub struct S3Path<'i, C: Into<BorrowedS3PathComp<'i>>, I: S3PathIter<C>> {
    components: I,
    phantom_data: PhantomData<&'i ()>,
    phantom_data2: PhantomData<C>,
}

impl<'i, C: Into<BorrowedS3PathComp<'i>>, I: S3PathIter<C>> S3Path<'i, C, I> {
    pub fn try_from(components: I) -> Result<Self, InvalidS3PathComponent> {
        for component in components.clone().into_iter() {
            validation::validate_component(&component.into())?;
        }
        Ok(Self {
            components,
            phantom_data: PhantomData,
            phantom_data2: PhantomData,
        })
    }

    fn new_unchecked(components: I) -> Self {
        Self {
            components,
            phantom_data: PhantomData,
            phantom_data2: PhantomData,
        }
    }

    pub fn to_owned(self) -> S3PathBuf {
        S3PathBuf {
            components: self
                .components
                .into_iter()
                .map(|it| Cow::Owned(it.into().to_string()))
                .collect(),
        }
    }
}

impl<'i, C: Into<BorrowedS3PathComp<'i>>, I: S3PathIter<C>> std::fmt::Display for S3Path<'i, C, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_components(self.components.clone().into_iter().map(|it| it.into()), f)?;
        Ok(())
    }
}

impl<'i, C: Into<BorrowedS3PathComp<'i>>, I: S3PathIter<C>> std::fmt::Debug for S3Path<'i, C, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_components(self.components.clone().into_iter().map(|it| it.into()), f)?;
        Ok(())
    }
}

/// An owned S3 storage path.
#[derive(Clone, PartialEq, Eq, Default)]
pub struct S3PathBuf {
    components: Vec<OwnedS3PathComp>,
}

impl S3PathBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_from<'i, C: Into<OwnedS3PathComp>, I: S3PathIter<C>>(
        components: I,
    ) -> Result<Self, InvalidS3PathComponent> {
        let mut path = S3PathBuf::new();
        for component in components.into_iter() {
            path.join(component)?;
        }
        Ok(path)
    }

    pub fn try_from_str(string: impl AsRef<str>) -> Result<Self, InvalidS3PathComponent> {
        let mut path = S3PathBuf::new();
        for c in string.as_ref().split('/') {
            // Skip empty components from consecutive slashes
            if !c.is_empty() {
                path.join(Cow::Owned(c.to_string()))?;
            }
        }
        Ok(path)
    }

    pub fn join(
        &mut self,
        component: impl Into<Cow<'static, str>>,
    ) -> Result<&mut Self, InvalidS3PathComponent> {
        let component_str = component.into();
        validation::validate_component(component_str.as_ref())?;
        self.components.push(component_str);
        Ok(self)
    }

    pub fn as_path(&self) -> S3Path<Cow<str>, impl S3PathIter<Cow<str>>> {
        // SAFETY: We can use `new_unchecked` here,
        // because components were already validated when added!
        S3Path::new_unchecked(self.components.iter().map(|it| Cow::Borrowed(it.as_ref())))
    }

    pub fn to_std_path_buf(&self) -> PathBuf {
        let mut path = PathBuf::new();
        for c in &self.components {
            path = path.join(c.as_ref());
        }
        path
    }
}

impl TryFrom<&str> for S3PathBuf {
    type Error = InvalidS3PathComponent;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let mut path = S3PathBuf::new();
        for c in value.split('/') {
            // Skip empty components from consecutive slashes
            if !c.is_empty() {
                path.join(Cow::Owned(c.to_string()))?;
            }
        }
        Ok(path)
    }
}

impl TryFrom<String> for S3PathBuf {
    type Error = InvalidS3PathComponent;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        TryFrom::try_from(value.as_str())
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

#[cfg(test)]
mod test {
    mod s3_path_buf {
        use crate::S3PathBuf;
        use assertr::prelude::*;

        #[test]
        fn new_is_initially_empty() {
            let path = S3PathBuf::new();
            assert_that(path).has_display_value("");
        }

        #[test]
        fn construct_using_new_and_join_components() {
            let mut path = S3PathBuf::new();
            path.join("foo").unwrap();
            path.join("bar").unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn construct_using_try_from_given_str() {
            let path = S3PathBuf::try_from_str("foo/bar").unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn construct_using_try_from_given_string() {
            let path = S3PathBuf::try_from_str("foo/bar".to_string()).unwrap();
            assert_that(path).has_display_value("foo/bar");
        }

        #[test]
        fn reject_invalid_characters() {
            let mut path = S3PathBuf::new();
            let result = path.join("invalid/path");
            assert_that(result.is_err()).is_true();

            let result = S3PathBuf::try_from_str("foo/bar$baz");
            assert_that(result.is_err()).is_true();
        }
    }

    mod s3_path {

        mod new {
            use crate::S3Path;
            use assertr::prelude::*;

            #[test]
            fn from_static_str_array() {
                let path = S3Path::try_from(["foo", "bar"]).unwrap();
                assert_that(path).has_display_value("foo/bar");
            }

            #[test]
            fn from_borrowed_str_array() {
                let components = ["foo".to_string(), "bar".to_string()];
                let components_ref = [components[0].as_str(), components[1].as_str()];
                let path = S3Path::try_from(components_ref).unwrap();
                assert_that(path).has_display_value("foo/bar");
            }

            #[test]
            fn from_string_array() {
                let path = S3Path::try_from(["foo".to_string(), "bar".to_string()]).unwrap();
                assert_that(path).has_display_value("foo/bar");
            }

            #[test]
            fn reject_invalid_characters() {
                assert_that(S3Path::try_from(["foo-bar"]))
                    .is_ok()
                    .has_display_value("foo-bar");
                assert_that(S3Path::try_from(["foo_bar"]))
                    .is_ok()
                    .has_display_value("foo_bar");
                assert_that(S3Path::try_from(["foo.bar"]))
                    .is_ok()
                    .has_display_value("foo.bar");
                assert_that(S3Path::try_from([".test"]))
                    .is_ok()
                    .has_display_value(".test");

                assert_that(S3Path::try_from(["foo$bar"])).is_err();
                assert_that(S3Path::try_from(["foo&bar"])).is_err();
                assert_that(S3Path::try_from(["foo#bar"])).is_err();
                assert_that(S3Path::try_from(["foo/bar"])).is_err();
                assert_that(S3Path::try_from(["foo|bar"])).is_err();
                assert_that(S3Path::try_from(["foo\\bar"])).is_err();
                assert_that(S3Path::try_from(["."])).is_err();
                assert_that(S3Path::try_from([".."])).is_err();
            }
        }
    }
}

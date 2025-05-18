use std::borrow::Cow;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::path::PathBuf;

pub type S3PathComp<'i> = Cow<'i, str>;

pub trait S3PathIter<'i>: IntoIterator<Item = S3PathComp<'i>> + Clone {}

impl<'i, T: IntoIterator<Item = S3PathComp<'i>> + Clone> S3PathIter<'i> for T {}

#[derive(Clone)]
pub struct S3Path<'i, I: S3PathIter<'i>> {
    components: I,
    phantom_data: PhantomData<&'i ()>,
}

impl<'i, I: S3PathIter<'i>> S3Path<'i, I> {
    pub fn new(components: I) -> Self {
        Self {
            components,
            phantom_data: PhantomData,
        }
    }
}

impl<'i, I: S3PathIter<'i>> std::fmt::Display for S3Path<'i, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_components(self.components.clone().into_iter(), f)?;
        Ok(())
    }
}

impl<'i, I: S3PathIter<'i>> std::fmt::Debug for S3Path<'i, I> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write_components(self.components.clone().into_iter(), f)?;
        Ok(())
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct S3PathBuf {
    components: Vec<String>,
}

impl S3PathBuf {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn join(&mut self, component: impl Into<String>) -> &mut Self {
        self.components.push(component.into());
        self
    }

    pub fn as_path(&self) -> S3Path<impl S3PathIter<'_>> {
        S3Path::new(self.components.iter().map(|it| Cow::Borrowed(it.as_str())))
    }

    pub fn to_std_path_buf(&self) -> PathBuf {
        let mut path = PathBuf::new();
        for c in &self.components {
            path = path.join(c);
        }
        path
    }
}

impl<T: AsRef<str>> From<T> for S3PathBuf {
    fn from(value: T) -> Self {
        let mut path = S3PathBuf::new();
        for c in value.as_ref().split('/') {
            path.join(c);
        }
        path
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
    use crate::{S3Path, S3PathBuf};
    use assertr::prelude::*;
    use std::borrow::Cow;

    #[test]
    fn construct_owned_path() {
        let mut path = S3PathBuf::new();
        path.join("foo");
        path.join("bar");
        assert_that(path.to_string()).is_equal_to("foo/bar");
    }

    #[test]
    fn construct_borrowed_path() {
        let path = S3Path::new([Cow::Borrowed("foo"), Cow::Borrowed("bar")]);
        assert_that(path.to_string()).is_equal_to("foo/bar");
    }
}

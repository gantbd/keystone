//! Diagnostic wrappers that make disclosure an explicit operation.

use std::fmt;

#[derive(Clone, Eq, PartialEq)]
pub struct Secret<T>(T);

impl<T> Secret<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn expose(&self) -> &T {
        &self.0
    }

    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Secret<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Secret([REDACTED])")
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Content<T> {
    value: T,
    bytes: usize,
}

impl<T> Content<T> {
    pub fn new(value: T, bytes: usize) -> Self {
        Self { value, bytes }
    }

    pub fn expose(&self) -> &T {
        &self.value
    }

    pub fn byte_len(&self) -> usize {
        self.bytes
    }

    pub fn into_inner(self) -> T {
        self.value
    }
}

impl<T> fmt::Debug for Content<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Content")
            .field("bytes", &self.bytes)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_never_reveals_wrapped_values() {
        let secret = Secret::new("credential-value");
        let content = Content::new("private prompt", 14);
        assert!(!format!("{secret:?}").contains("credential-value"));
        assert!(!format!("{content:?}").contains("private prompt"));
        assert!(format!("{content:?}").contains("14"));
    }
}

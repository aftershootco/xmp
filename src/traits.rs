pub trait UnrwrapOrErr<T> {
    fn unrwrap_or_err<F: FnOnce() -> E, E: std::error::Error>(self, f: F) -> Result<T, E>;
}

impl<T> UnrwrapOrErr<T> for Option<T> {
    fn unrwrap_or_err<F: FnOnce() -> E, E: std::error::Error>(self, f: F) -> Result<T, E> {
        if let Some(t) = self {
            Ok(t)
        } else {
            Err(f())
        }
    }
}

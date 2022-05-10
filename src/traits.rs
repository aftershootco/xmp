pub trait OtoR<T> {
    fn otor<F: FnOnce() -> E, E: std::error::Error>(self, f: F) -> Result<T, E>;
}

impl<T> OtoR<T> for Option<T> {
    fn otor<F: FnOnce() -> E, E: std::error::Error>(self, f: F) -> Result<T, E> {
        if let Some(t) = self {
            Ok(t)
        } else {
            Err(f())
        }
    }
}

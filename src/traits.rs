use minidom::IntoAttributeValue;
use std::collections::HashMap;
use std::hash::Hash;
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

pub trait PrefixNS {
    type Error;
    fn add_prefix<P, V>(&mut self, prefix: P, value: V) -> Result<(), Self::Error>
    where
        P: Into<String> + Hash + Eq,
        V: IntoAttributeValue + Hash + Eq;
    fn add_prefixes<P, V, I, A>(&mut self, prefixes: A) -> Result<(), Self::Error>
    where
        P: Into<String>,
        V: IntoAttributeValue,
        I: Iterator<Item = (P, V)>,
        A: IntoIterator<IntoIter = I>;
}

impl PrefixNS for minidom::Element {
    type Error = crate::errors::XmpError;
    fn add_prefix<P, V>(&mut self, prefix: P, value: V) -> Result<(), Self::Error>
    where
        P: Into<String> + std::hash::Hash + Eq,
        V: IntoAttributeValue + std::hash::Hash + Eq,
    {
        self.add_prefixes([(prefix, value)])
    }

    fn add_prefixes<P, V, I, A>(&mut self, prefixes: A) -> Result<(), Self::Error>
    where
        P: Into<String>,
        V: IntoAttributeValue,
        I: Iterator<Item = (P, V)>,
        A: IntoIterator<IntoIter = I>,
    {
        let sets: HashMap<String, V> = prefixes.into_iter().map(|(p, v)| (p.into(), v)).collect();

        for (prefix, value) in sets {
            // self.prefixes.get(&Some(prefix.clone()));
            if let Some(value) = value.into_attribute_value() {
                self.prefixes.insert(Some(dbg!(prefix)), dbg!(value));
            }
        }
        dbg!(&self.prefixes);
        Ok(())
    }
}

use std::fmt;

#[derive(Clone)]
pub struct List<T: fmt::Display>(Vec<T>);
impl<T: fmt::Display> std::ops::Deref for List<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: fmt::Display> std::ops::DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: fmt::Display> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<T: fmt::Display> fmt::Display for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut comma_separated = String::new();
        for num in &self.0[0..self.0.len() - 1] {
            comma_separated.push_str(&num.to_string());
            comma_separated.push_str(", ");
        }
        comma_separated.push_str(&self.0[self.0.len() - 1].to_string());
        write!(f, "{}", comma_separated)
    }
}

impl<T: fmt::Display> List<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}


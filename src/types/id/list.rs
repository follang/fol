use std::fmt;

#[derive(Clone)]
pub struct List<T: Clone + fmt::Display>(Vec<T>);
impl<T: Clone + fmt::Display> std::ops::Deref for List<T> {
    type Target = Vec<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T: Clone + fmt::Display> std::ops::DerefMut for List<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Clone + fmt::Display> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<T: Clone + fmt::Display> fmt::Display for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut comma_separated = String::new();
        for num in &self.0[0..self.0.len() - 1] {
            comma_separated.push_str(&num.to_string());
            comma_separated.push_str("; ");
        }
        comma_separated.push_str(&self.0[self.0.len() - 1].to_string());
        write!(f, "{}", comma_separated)
    }
}

impl<T: Clone + fmt::Display> List<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get(&self, num: usize) -> T {
        self.0[num].clone()
    }

    pub fn print(&self) -> String {
        let mut space_separated = String::new();
        for num in &self.0[0..self.0.len() - 1] {
            space_separated.push_str(&num.to_string());
            space_separated.push_str(" ");
        }
        space_separated.push_str(&self.0[self.0.len() - 1].to_string());
        format!("{}", space_separated)
    }
}


use crate::syntax::point;
use crate::syntax::token;

#[derive(Debug, Clone)]
pub struct ID<T: ?Sized + std::fmt::Display> {
    pub key: Option<token::KEYWORD>,
    pub loc: Option<point::Location>,
    pub node: T,
}

impl<T: std::fmt::Display> ID<T> {
    pub fn new(node: T) -> Self {
        Self{ key: None, loc: None, node }
    }
    pub fn key(&self) -> Option<token::KEYWORD> {
        self.key.clone()
    }
    pub fn set_key(&mut self, key: token::KEYWORD) {
        self.key = Some(key);
    }
    pub fn loc(&self) -> Option<point::Location> {
        self.loc.clone()
    }
    pub fn set_loc(&mut self, loc: point::Location) {
        self.loc = Some(loc);
    }
    pub fn node(&self) -> &T {
        &self.node
    }
    pub fn set_node(&mut self, node: T) {
        self.node = node
    }
}

impl<T: std::fmt::Display> std::fmt::Display for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let loc = match self.loc() { Some(e) => e.to_string(), None => String::new()  };
        write!(f, "{}\t{}", loc, self.node())
    }
}

impl<T: std::fmt::Display> std::ops::Deref for ID<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}
impl<T: std::fmt::Display> std::ops::DerefMut for ID<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}

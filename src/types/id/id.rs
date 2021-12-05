use crate::syntax::point;

#[derive(Debug, Clone)]
pub struct ID<T: ?Sized + std::fmt::Display> {
    pub loc: Option<point::Location>,
    pub node: T,
}

impl<T: std::fmt::Display> ID<T> {
    pub fn new(node: T) -> Self {
        Self{ loc: None, node }
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
        let mut loc = 0;
        if let Some(l) = &self.loc() { loc = (l.deep()+1)*2; }
        write!(f, "{}{}", " ".repeat(loc as usize), self.node())
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

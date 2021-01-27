use crate::syntax::point;

#[derive(Debug, Clone)]
pub struct ID<T: ?Sized + std::fmt::Display> {
    pub loc: Option<point::Location>,
    pub node: T,
}

impl<T: std::fmt::Display> ID<T> {
    pub fn new(loc: Option<point::Location>, node: T) -> Self {
        Self{ loc, node: node }
    }
    pub fn get_loc(&self) -> Option<point::Location> {
        self.loc.clone()
    }
    pub fn set_loc(&mut self, loc: point::Location) {
        self.loc = Some(loc);
    }
    pub fn get_node(&self) -> &T {
        &self.node
    }
    pub fn set_node(&mut self, node: T) {
        self.node = node
    }
}

impl<T: std::fmt::Display> std::fmt::Display for ID<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let loc = match self.get_loc() { Some(e) => e.to_string(), None => String::new()  };
        write!(f, "{}\t{}", loc, self.get_node())
    }
}

#[derive(Clone)] 
pub enum Neo<T> {
    None,
    One(T),
    Many(Vec<T>),
}

impl<T: Clone> Neo<T> {
    #[inline]
    pub fn is_one(&self) -> bool {
        matches!(*self, Neo::One(_))
    }
    #[inline]
    pub fn is_many(&self) -> bool {
        !self.is_one()
    }
    pub fn pop(&mut self) -> Option<T> {
        match self {
            Neo::One(v) => { 
                let elem = v.clone(); 
                *self = Self::None;
                Some(elem)  
            },
            Neo::Many(v) => { 
                let mut vec = v.clone();
                let elem = vec.pop().unwrap();
                if vec.len() > 1 {
                    *self = Self::Many(vec);
                } else if vec.len() == 1 {
                    *self = Self::One(vec.pop().unwrap())
                } else {
                    *self = Self::None
                }
                Some(elem)
            },
            Neo::None => { 
                None 
            },
        }
    }
    pub fn as_vec(self) -> Vec<T> {
        match self { 
            Neo::One(v) => vec![v.clone()],
            Neo::Many(v) => v.clone(),
            Neo::None => Vec::new()
        }
    }
}


impl<T: Clone> Iterator for Neo<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}

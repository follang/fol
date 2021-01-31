use std::cell::RefCell;

pub struct Pool<T> {
    items: RefCell<Vec<T>>
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        Pool{ items: RefCell::new(Vec::new()) }
    }

    pub fn get(&self) -> Option<PoolGuard<T>> {
        let item = match self.items.borrow_mut().pop() {
            Some(val) => val,
            None => return None
        };
        Some(PoolGuard { inner: Some(item), items: &self.items })
    }
    pub fn push(&self, item: T) {
        self.items.borrow_mut().push(item);
    }
    pub fn pop(&self, item: T) {
        self.items.borrow_mut().pop();
    }
    pub fn iter(&self) -> impl Iterator<Item = T> + '_ {
        let mut items = self.items.borrow_mut();
        std::iter::from_fn(move || {
            items.pop()
        })
    }


}

pub struct PoolGuard<'a, T> {
    inner: Option<T>,
    items: &'a RefCell<Vec<T>>
}

impl<T> Drop for PoolGuard<'_, T> {
    fn drop(&mut self) {
        let item = self.inner.take().unwrap();
        &self.items.borrow_mut().push(item);
    }
}

impl<T> std::ops::Deref for PoolGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner.as_ref().unwrap()
    }
}
impl<T> std::ops::DerefMut for PoolGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut().unwrap()
    }
}

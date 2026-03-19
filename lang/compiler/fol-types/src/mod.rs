// Core type definitions
pub const SLIDER: usize = 9;
pub type Win<T> = (Vec<T>, T, Vec<T>);
pub type Con<T> = Result<T, Box<dyn crate::Glitch + 'static>>;
pub type Vod = Result<(), Box<dyn crate::Glitch + 'static>>;
pub type List<T> = Vec<T>;
pub type Errors = Vec<Box<dyn crate::Glitch>>;

#[macro_export]
macro_rules! catch {
    ($err:expr $(,)?) => {{
        Box::new($err)
    }};
}


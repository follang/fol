#![allow(dead_code)]
#![allow(unused_macros)]

pub const SLIDER: usize = 9;
pub const EOF_CHAR: char = '\0';
pub type Win<T> = (Vec<T>, T, Vec<T>);


#[macro_export]
macro_rules! catch {
    ($err:expr $(,)?) => ({ Box::new($err) });
}
#[macro_export]
macro_rules! crash {
    () => ({ std::process::exit(0); });
    ($err:expr $(,)?) => ({ println!("{}", $err); std::process::exit(0); });
}

pub trait Glitch: std::error::Error {}

pub type Con<T> = Result<T, Box<(dyn Glitch + 'static)>>;
pub type Vod = Result<(), Box<(dyn Glitch + 'static)>>;
pub type Opt<T> = Option<T>;

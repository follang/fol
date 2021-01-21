#![allow(dead_code)]
#![allow(unused_macros)]

use crate::syntax::error::Glitch;


pub const SLIDER: usize = 9;
pub const EOF_CHAR: char = '\0';
pub type Win<T> = (Vec<T>, T, Vec<T>);

#[macro_export]
macro_rules! catch {
    ($err:expr $(,)?) => ({ Err($err) });
}
#[macro_export]
macro_rules! crash {
    () => ({ std::process::exit(0); });
    ($err:expr $(,)?) => ({ println!("{}", $err); std::process::exit(0); });
}


pub type Con<T> = Result<T, Glitch>;
pub type Vod = Result<(), Glitch>;

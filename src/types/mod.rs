#![allow(dead_code)]
#![allow(unused_macros)]

pub mod error;
use crate::types::error::{Glitch, Fault};


pub const SLIDER: usize = 9;
pub type Win<T> = (Vec<T>, T, Vec<T>);

macro_rules! flaw {
    ($err:expr $(,)?) => ({ Fault::Flaw( $err ) });
}

macro_rules! typo {
    ($err:expr $(,)?) => ({ Fault::Typo( $err ) });
}

macro_rules! slip {
    ($err:expr $(,)?) => ({ Fault::Slip( $err ) });
}

macro_rules! catch {
    ($err:expr $(,)?) => ({ Box::new($err) });
}

#[macro_export]
macro_rules! crash {
    () => ({ std::process::exit(0); });
    ($err:expr $(,)?) => ({ println!("{}", $err); std::process::exit(0); });
}


pub type Con<T> = Result<T, Box<(dyn Glitch + 'static)>>;
// pub type Con<T> = Result<T, Fault>;
pub type Vod = Result<(), Box<(dyn Glitch + 'static)>>;
// pub type Vod = Result<(), Fault>;

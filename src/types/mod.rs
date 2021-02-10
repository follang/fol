#![allow(dead_code)]
#![allow(unused_macros)]

// use colored::Colorize;
// use terminal_size::{Width, Height, terminal_size};

pub mod error;
pub use crate::types::error::*;

pub mod id;
pub use crate::types::id::*;

pub const SLIDER: usize = 9;
pub type Win<T> = (Vec<T>, T, Vec<T>);
pub type Con<T> = Result<T, Box<(dyn Glitch + 'static)>>;
pub type Vod = Result<(), Box<(dyn Glitch + 'static)>>;

// impl<T> From<Con<T>> for Vod {
//     fn from(stat: Con<T>) -> Self {
//         if let Err(e) = stat {
//             return Err(e)
//         } else { return () }
//     }
// }

#[macro_export]
macro_rules! catch {
    ($err:expr $(,)?) => ({ Box::new($err) });
}

#[macro_export]
macro_rules! crash {
    () => ({ std::process::exit(0); });
    ($err:expr $(,)?) => ({ println!("{}", $err); std::process::exit(0); });
}

#[macro_export]
macro_rules! halt {
    () => ({ println!("\n ... UNIMPLEMENTED ... \n"); std::process::exit(0); });
}


#[macro_export]
macro_rules! printer {
    ($err:expr $(,)?) => ({ for e in $err { println!("{}", e)} });
}

#[macro_export]
macro_rules! logit {
    ($arg:expr) => ({
        use terminal_size::{Width, Height, terminal_size};
        let mut width = if let Some((Width(w), Height(h))) = terminal_size() { w as usize } else { 20 };
        width = width - $arg.len();
        use colored::Colorize;
        let p = format!("\n{}  {}  {}", "=".repeat(5 - 2), $arg.red(), "=".repeat(width - 5 - 2));
        println!("{}", p); 
    });
}

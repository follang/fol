#![allow(dead_code)]
#![allow(unused_macros)]

// Core type definitions
pub const SLIDER: usize = 9;
pub type Win<T> = (Vec<T>, T, Vec<T>);
pub type Con<T> = Result<T, Box<(dyn crate::Glitch + 'static)>>;
pub type Vod = Result<(), Box<(dyn crate::Glitch + 'static)>>;
pub type List<T> = Vec<T>;
pub type Errors = Vec<Box<dyn crate::Glitch>>;

#[macro_export]
macro_rules! catch {
    ($err:expr $(,)?) => {{
        Box::new($err)
    }};
}

#[macro_export]
macro_rules! crash {
    () => {{
        std::process::exit(0);
    }};
    ($err:expr $(,)?) => {{
        println!("{}", $err);
        std::process::exit(0);
    }};
}

#[macro_export]
macro_rules! halt {
    () => {{
        println!("\n ... UNIMPLEMENTED ... \n");
        std::process::exit(0);
    }};
}

#[macro_export]
macro_rules! erriter {
    ($err:expr $(,)?) => {{
        for e in $err.iter().enumerate() {
            let bup = border_up("-", " FLAW: #".to_string() + &e.0.to_string() + " ");
            println!("{}{}", bup, e.1)
        }
    }};
}

#[macro_export]
macro_rules! noditer {
    ($nod:expr $(,)?) => {{
        for e in $nod {
            println!("{}", e);
        }
    }};
}

#[macro_export]
macro_rules! logit {
    ($arg:expr) => {{
        use terminal_size::{terminal_size, Height, Width};
        let mut width = if let Some((Width(w), Height(h))) = terminal_size() {
            w as usize
        } else {
            20
        };
        width = width - $arg.len();
        use colored::Colorize;
        let p = format!(
            "\n{}  {}  {}",
            "=".repeat(5 - 2),
            $arg.red(),
            "=".repeat(width - 5 - 2)
        );
        println!("{}", p);
    }};
}

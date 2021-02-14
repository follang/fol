pub mod source;
pub mod elements;

pub use crate::syntax::index::source::{Source, Sources};
pub use crate::syntax::index::elements::Elements;

pub enum SourceType {
    Soruce(Source),
    String(String)
}

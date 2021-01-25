use dyn_clone::DynClone;

pub mod assign;
pub mod r#type;
pub use crate::syntax::nodes::{NodeTrait, id};

pub use crate::syntax::nodes::opts::{assign::AssOpts, r#type::TypOpts};


pub trait OptsTrait: NodeTrait {}
dyn_clone::clone_trait_object!(OptsTrait);

pub type Opts = id<Box<dyn OptsTrait + 'static>>;

#![allow(clippy::match_same_arms)]
#![feature(iterator_try_collect)]
#![feature(decl_macro)]

mod model;
pub mod proto;

use std::error::Error;

pub use model::*;

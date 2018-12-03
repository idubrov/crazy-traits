#![feature(plugin)]
#![plugin(dynasm)]
extern crate dynasmrt;
extern crate typed_arena;

pub mod arena;
pub mod dynvtable;
pub mod leaking;

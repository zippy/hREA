#![feature(try_from)]
/**
 * Observation module datatypes & behaviours
 */

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

#[macro_use]
extern crate vf_core;

pub use vf_core::{ type_aliases, measurement };

pub mod economic_event;

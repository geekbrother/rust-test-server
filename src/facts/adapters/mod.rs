//! Adapters for transforming facts endpoint result into the our server expected
//! format. Adapters should extract the fact and return it as a string.
//!
//! Adapters should implement the `Transformable` trait from the `facts` parent module.

pub mod cat;
pub mod dog;

use crate::{
    event::{Event,Value},
    vicscript::Result
};
use string_cache::DefaultAtom as Atom;

pub trait Function: Send + core::fmt::Debug {
    fn execute(&self, context: &Event) -> Result<Value>;
}

//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Literal {
    value: Value
}

impl From<Value> for Literal {
    fn from(value: Value) -> Self {
        Literal{value}
    }
}

impl Function for Literal {
    fn execute(&self, _: &Event) -> Result<Value> {
        Ok(self.value.clone())
    }
}

//------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Path {
    path: Atom
}

impl Path {
    pub fn new(target: &str) -> Self {
        Self{
            path: Atom::from(target),
        }
    }
}

impl Function for Path {
    fn execute(&self, ctx: &Event) -> Result<Value> {
        ctx.as_log().get(&self.path).cloned().ok_or(format!("path {} not found in event", self.path))
    }
}
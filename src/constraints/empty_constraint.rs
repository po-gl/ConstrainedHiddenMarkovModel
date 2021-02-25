use crate::constraints::Constraint;
use std::any::Any;
use std::fmt::{Formatter, Error};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct EmptyConstraint { }

impl EmptyConstraint {
    pub fn new() -> EmptyConstraint {
        EmptyConstraint {}
    }
}

impl Constraint for EmptyConstraint {
    fn is_satisfied_by_state(&self, _state: String) -> bool {
        true
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn box_eq(&self, other: &dyn Any) -> bool {
        other.downcast_ref::<Self>().map_or(false, |a| self == a)
    }

    fn debug_fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Box:{:?}", self)
    }

    fn constraint_clone(&self) -> Box<dyn Constraint + Send> {
        Box::new(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_empty_constraint() {
        let constraint = EmptyConstraint::new();
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("Anything")))
    }
}
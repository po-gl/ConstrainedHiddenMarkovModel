use crate::constraints::Constraint;
use std::any::Any;
use std::fmt::{Formatter, Error};

#[derive(Debug, PartialEq, Clone)]
pub struct MatchesConstraint {
    pub state: String
}

impl MatchesConstraint {
    pub fn new(state: String) -> MatchesConstraint {
        MatchesConstraint {
            state: state.to_lowercase()
        }
    }
}

impl Constraint for MatchesConstraint {
    fn is_satisfied_by_state(&self, state: String) -> bool {
        state.to_lowercase() == self.state
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
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_matches_constraint() {
        let constraint = MatchesConstraint::new(String::from("FrEd"));
        assert_eq!("fred", constraint.state)
    }

    #[test]
    fn satisfying_matches_constraint() {
        let constraint = MatchesConstraint::new(String::from("mary"));
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("mary")))
    }

    #[test]
    fn not_satisfying_matches_constraint() {
        let constraint = MatchesConstraint::new(String::from("Mary"));
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("Marge")))
    }

    #[test]
    fn empty_satisfying_matches_constraint() {
        let constraint = MatchesConstraint::new(String::from("Barry"));
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("")))
    }
}
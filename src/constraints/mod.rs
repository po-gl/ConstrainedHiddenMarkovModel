pub(crate) mod starts_with_letter_constraint;
pub(crate) mod empty_constraint;
pub(crate) mod matches_constraint;
pub(crate) mod multi_constraint;
pub(crate) mod rhymes_with_constraint;

use std::any::Any;
use std::fmt::{Formatter, Error, Debug};

pub trait Constraint: Any {
    // Constraint functions
    fn is_satisfied_by_state(&self, state: String) -> bool;

    // Functions to facilitate dynamic typing
    fn as_any(&self) -> &dyn Any;
    fn box_eq(&self, other: &dyn Any) -> bool;
    fn debug_fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
    fn constraint_clone(&self) -> Box<dyn Constraint + Send>;
}

impl PartialEq for Box<dyn Constraint + Send> {
    fn eq(&self, other: &Box<dyn Constraint + Send>) -> bool {
        self.box_eq(other.as_any())
    }
}

impl Debug for Box<dyn Constraint + Send> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        self.debug_fmt(f)
    }
}

impl Clone for Box<dyn Constraint + Send> {
    fn clone(&self) -> Self {
        self.constraint_clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::empty_constraint::EmptyConstraint;
    use crate::constraints::starts_with_letter_constraint::StartsWithLetterConstraint;
    use crate::constraints::matches_constraint::MatchesConstraint;

    #[test]
    fn create_dynamic_type_constraint() {
        let mut dynamic_constraint: Box<dyn Constraint + Send>;
        dynamic_constraint = Box::new(EmptyConstraint::new());
        assert_eq!(true, dynamic_constraint.is_satisfied_by_state(String::from("foo")));

        dynamic_constraint = Box::new(StartsWithLetterConstraint::new('x'));
        assert_eq!(false, dynamic_constraint.is_satisfied_by_state(String::from("foo")));
    }
    
    #[test]
    fn dynamic_type_constraint_array() {
        let mut dynamic_constraints: Vec<Box<dyn Constraint + Send>> = vec![];
        dynamic_constraints.push(Box::new(StartsWithLetterConstraint::new('f')));
        dynamic_constraints.push(Box::new(EmptyConstraint::new()));
        dynamic_constraints.push(Box::new(MatchesConstraint::new(String::from("george"))));
        dynamic_constraints.push(Box::new(EmptyConstraint::new()));
        dynamic_constraints.push(Box::new(EmptyConstraint::new()));
        dynamic_constraints.push(Box::new(StartsWithLetterConstraint::new('m')));

        assert_eq!(6, dynamic_constraints.len());
        assert_eq!(true, dynamic_constraints[0].is_satisfied_by_state(String::from("Food")));
        assert_eq!(true, dynamic_constraints[1].is_satisfied_by_state(String::from("")));
        assert_eq!(false, dynamic_constraints[2].is_satisfied_by_state(String::from("gorge")));
        assert_eq!(true, dynamic_constraints[3].is_satisfied_by_state(String::from("12345")));
        assert_eq!(true, dynamic_constraints[4].is_satisfied_by_state(String::from("   ")));
        assert_eq!(false, dynamic_constraints[5].is_satisfied_by_state(String::from("Ned")));
    }
}
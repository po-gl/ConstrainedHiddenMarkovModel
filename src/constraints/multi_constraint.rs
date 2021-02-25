use crate::constraints::Constraint;
use std::any::Any;
use std::fmt::{Formatter, Error};

#[derive(Debug, PartialEq, Clone)]
pub struct MultiConstraint {
    pub constraints: Vec<Box<dyn Constraint + Send>>,
    pub require_all: bool
}

impl MultiConstraint {
    pub fn new(constraints: Vec<Box<dyn Constraint + Send>>, require_all: bool) -> MultiConstraint {
        MultiConstraint {
            constraints,
            require_all
        }
    }
}

impl Constraint for MultiConstraint {
    fn is_satisfied_by_state(&self, word: String) -> bool {
        if self.require_all {  // satisfies all constraints
            for constraint in self.constraints.iter() {
                if !constraint.is_satisfied_by_state(String::from(&word)) {
                    return false;
                }
            }
            return true;
        } else {  // satisfies any constraint
            for constraint in self.constraints.iter() {
                if constraint.is_satisfied_by_state(String::from(&word)) {
                    return true;
                }
            }
            return false;
        }
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
    use crate::constraints::starts_with_letter_constraint::StartsWithLetterConstraint;
    use crate::constraints::matches_constraint::MatchesConstraint;

    #[test]
    fn new_multi_constraint() {
        let constraint = MultiConstraint::new(vec![
            Box::new(StartsWithLetterConstraint::new('f')),
            Box::new(StartsWithLetterConstraint::new('t'))
        ], false);
        assert_eq!(2, constraint.constraints.len());
        assert_eq!(false, constraint.require_all);
    }

    #[test]
    fn satisfying_any_multi_constraint() {
        let constraint = MultiConstraint::new(vec![
            Box::new(StartsWithLetterConstraint::new('x')),
            Box::new(StartsWithLetterConstraint::new('z')),
            Box::new(StartsWithLetterConstraint::new('a')),
        ], false);
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("Xylophone")));
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("zebra")));
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("Apple")));
    }

    #[test]
    fn not_satisfying_any_multi_constraint() {
        let constraint = MultiConstraint::new(vec![
            Box::new(StartsWithLetterConstraint::new('x')),
            Box::new(StartsWithLetterConstraint::new('z')),
            Box::new(StartsWithLetterConstraint::new('a')),
        ], false);
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("Beaver")));
    }

    #[test]
    fn satisfying_all_multi_constraint() {
        let constraint = MultiConstraint::new(vec![
            Box::new(StartsWithLetterConstraint::new('x')),
            Box::new(MatchesConstraint::new(String::from("Xylo"))),
        ], true);
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("Xylo")));
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("X-ray")));
    }

    #[test]
    fn not_satisfying_all_multi_constraint() {
        let constraint = MultiConstraint::new(vec![
            Box::new(StartsWithLetterConstraint::new('x')),
            Box::new(StartsWithLetterConstraint::new('z')),
            Box::new(StartsWithLetterConstraint::new('a')),
        ], true);
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("Xylophone")));
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("zebra")));
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("Apple")));
    }
}
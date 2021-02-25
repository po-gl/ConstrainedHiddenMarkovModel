use crate::constraints::Constraint;
use std::any::Any;
use std::fmt::{Formatter, Error};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct StartsWithLetterConstraint {
    pub letter: char
}

impl StartsWithLetterConstraint {
    pub fn new(letter: char) -> StartsWithLetterConstraint {
        StartsWithLetterConstraint {
            letter
        }
    }
}

impl Constraint for StartsWithLetterConstraint {
    fn is_satisfied_by_state(&self, word: String) -> bool {
        return match word.chars().nth(0) {
            None => false,
            Some(first_letter) => first_letter.to_ascii_lowercase() == self.letter.to_ascii_lowercase()
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
        Box::new(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn new_starts_with_letter_constraint() {
        let constraint = StartsWithLetterConstraint::new('f');
        assert_eq!('f', constraint.letter)
    }

    #[test]
    fn satisfying_new_starts_with_letter_constraint() {
        let constraint = StartsWithLetterConstraint::new('x');
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("Xylophone")))
    }

    #[test]
    fn not_satisfying_new_starts_with_letter_constraint() {
        let constraint = StartsWithLetterConstraint::new('x');
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("zebra")))
    }

    #[test]
    fn empty_satisfying_new_starts_with_letter_constraint() {
        let constraint = StartsWithLetterConstraint::new('x');
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("")))
    }
}
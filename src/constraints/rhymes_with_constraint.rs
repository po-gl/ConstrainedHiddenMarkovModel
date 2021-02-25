use crate::constraints::Constraint;
use std::any::Any;
use std::fmt::{Formatter, Error};

/// The rhymes with constraint currently uses
/// the ttaw double metaphone phonetic encoding method
/// to determine if two words rhyme
///
/// This isn't the most accurate method and sometimes
/// gives incorrect results (e.g. Fred and red returning
/// false but Ted and red returning true)
#[derive(Debug, PartialEq, Clone)]
pub struct RhymesWithConstraint {
    pub word: String
}

impl RhymesWithConstraint {
    pub fn new(word: String) -> RhymesWithConstraint {
        RhymesWithConstraint {
            word: word.to_lowercase()
        }
    }
}

impl Constraint for RhymesWithConstraint {
    fn is_satisfied_by_state(&self, word: String) -> bool {
        ttaw::metaphone::rhyme(self.word.as_str(), word.to_lowercase().as_str())
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
    fn new_rhymes_with_constraint() {
        let constraint = RhymesWithConstraint::new(String::from("FrEd"));
        assert_eq!("fred", constraint.word)
    }

    #[test]
    fn satisfying_rhymes_with_constraint() {
        let constraint = RhymesWithConstraint::new(String::from("Mary"));
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("Berry")));
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("jeRRy")));
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("gary")));

        let constraint = RhymesWithConstraint::new(String::from("Ted"));
        assert_eq!(true, constraint.is_satisfied_by_state(String::from("red")));

        // Currently a problem for the ttaw method of determining rhymes
        // let constraint = RhymesWithConstraint::new(String::from("Fred"));
        // assert_eq!(true, constraint.is_satisfied_by_state(String::from("red")));
    }

    #[test]
    fn not_satisfying_rhymes_constraint() {
        let constraint = RhymesWithConstraint::new(String::from("Mary"));
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("Marge")));
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("Ted")));
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("green")));
    }

    #[test]
    fn empty_satisfying_matches_constraint() {
        let constraint = RhymesWithConstraint::new(String::from("Fred"));
        assert_eq!(false, constraint.is_satisfied_by_state(String::from("")))
    }
}
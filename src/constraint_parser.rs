use crate::{Constraint, EmptyConstraint, MatchesConstraint, StartsWithLetterConstraint};
use lazy_static::lazy_static;
use regex::Regex;
use crate::constraints::rhymes_with_constraint::RhymesWithConstraint;

pub(crate) fn parse_constraint(constraint_string: String) -> (Vec<Box<dyn Constraint + Send>>, Vec<Box<dyn Constraint + Send>>) {
    let mut hidden_constraints: Vec<Box<dyn Constraint + Send>> = vec![];
    let mut observed_constraints: Vec<Box<dyn Constraint + Send>> = vec![];

    for line in constraint_string.split("\n") {
        if line.is_empty() { continue; }
        if line.contains("*") {
            add_multi_constraint(line, &mut hidden_constraints, &mut observed_constraints);
        } else {
            add_constraint(line, &mut hidden_constraints, &mut observed_constraints);
        }
    }
    return (hidden_constraints, observed_constraints)
}

fn add_multi_constraint(line: &str, hidden: &mut Vec<Box<dyn Constraint + Send>>, observed: &mut Vec<Box<dyn Constraint + Send>>) {
    let mut line_split = line.split("*");
    let constraint_type = str_to_constraint(line_split.next().unwrap());
    let count: i32 = line_split.next().unwrap().parse().unwrap();
    for _ in 0..count { hidden.push(constraint_type.clone())}
    for _ in 0..count { observed.push(constraint_type.clone())}
}

fn add_constraint(line: &str, hidden_constraints: &mut Vec<Box<dyn Constraint + Send>>, observed_constraints: &mut Vec<Box<dyn Constraint + Send>>) {
    let mut line_split = line.split(":");
    let observed = line_split.next().unwrap();
    let hidden = line_split.next().unwrap();
    observed_constraints.push(str_to_constraint(observed));
    hidden_constraints.push(str_to_constraint(hidden));
}

// TODO: Support for multi-constraints
fn str_to_constraint(str: &str) -> Box<dyn Constraint + Send> {
    lazy_static! {
        static ref STARTS_WITH_RE: Regex = Regex::new(r"^SW\((.*)\)").unwrap();
        static ref RHYMES_WITH_RE: Regex = Regex::new(r"^RW\((.*)\)").unwrap();
        static ref EMPTY_RE: Regex = Regex::new(r"^NC").unwrap();
    }
    match STARTS_WITH_RE.captures(str) {
        Some(capture) => {
            let first_letter = capture[1].chars().nth(0).unwrap();
            return Box::new(StartsWithLetterConstraint::new(first_letter))
        },
        _ => (),
    }
    match RHYMES_WITH_RE.captures(str) {
        Some(capture) => return Box::new(RhymesWithConstraint::new(capture[1].to_string())),
        _ => (),
    }
    match EMPTY_RE.is_match(str) {
        true => return Box::new(EmptyConstraint::new()),
        false => (),
    }
    return Box::new(MatchesConstraint::new(str.to_string())); // default to match
}

#![allow(dead_code)]
mod console;
mod constrained_hidden_markov;
mod hidden_markov;
mod utils;
mod constraints;

use std::{env, fs};
use crate::utils::print_help;
use crate::constraints::multi_constraint::MultiConstraint;
use crate::constraints::Constraint;
use crate::constraints::starts_with_letter_constraint::StartsWithLetterConstraint;
use crate::constraints::empty_constraint::EmptyConstraint;
use crate::constraints::matches_constraint::MatchesConstraint;
use crate::hidden_markov::HiddenMarkov;
use crate::constrained_hidden_markov::ConstrainedHiddenMarkov;

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut training_file_path: String = "".to_string();
    for i in 1..args.len() {
        if args[i] == "-h" || args[i] == "help" || args[i] == "--help" {
            print_help();
            return;
        } else {
            training_file_path = args[i].clone();
        }
    }

    let observed_constraints: Vec<Box<dyn Constraint + Send>> = vec![
        Box::new(MultiConstraint::new(vec![
            Box::new(StartsWithLetterConstraint::new('t')),
            Box::new(StartsWithLetterConstraint::new('f')),
        ], false)),
        // Box::new(RhymesWithConstraint::new(String::from("red"))),
        Box::new(EmptyConstraint::new()),
        Box::new(EmptyConstraint::new()),
        Box::new(MatchesConstraint::new(String::from("red"))),
    ];
    let data = fs::read_to_string(training_file_path).expect("Unable to read data file");
    println!("Data length: {}", data.len());
    let model = HiddenMarkov::new(1, data);
    let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, Some(observed_constraints));
    constrained_model.train();

    println!("Sampled Sequences:");
    for _ in 0..10 {
        let str = constrained_model.sample_sequence(true);
        println!("{}", str);
        // println!("{}", constrained_model.get_sequence_probability(&*str));
    }
}

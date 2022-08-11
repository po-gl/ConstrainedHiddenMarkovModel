#![allow(dead_code)]

extern crate core;

use crate::constrained_hidden_markov::ConstrainedHiddenMarkov;
use crate::hidden_markov::HiddenMarkov;
use crate::console::{Args};
use crate::constraints::Constraint;
use crate::constraints::matches_constraint::MatchesConstraint;
use crate::constraints::empty_constraint::EmptyConstraint;
use crate::constraints::multi_constraint::MultiConstraint;
use crate::constraints::starts_with_letter_constraint::StartsWithLetterConstraint;
use std::time::Instant;
use crate::constraint_parser::parse_constraint;
use crate::utils::{get_data, print_sequences, write_sequences};

mod console;
mod constrained_hidden_markov;
mod hidden_markov;
mod utils;
mod constraints;
mod time_analysis;
mod constraint_parser;
mod config;

fn main() {
    let args = Args::new();

    let data = get_data(args.training_file);
    let (hidden_constraints, observed_constraints) = parse_constraint(args.constraint_string);

    let constrained_model = train_model(data, args.markov_order, hidden_constraints, observed_constraints);
    let sequences = generate_sequences(&constrained_model, args.num_of_sequences);

    if args.output_file.is_empty() {
        print_sequences(sequences);
    } else {
        write_sequences(sequences, args.output_file);
    }
}

fn train_model(data: String, markov_order: u32, hidden_constraints: Vec<Box<dyn Constraint + Send>>, observed_constraints: Vec<Box<dyn Constraint + Send>>) -> ConstrainedHiddenMarkov {
    let start = Instant::now();
    println!("Data length: {}\nSequence length: {}", data.len(), hidden_constraints.len());
    let model = HiddenMarkov::new(markov_order, data);
    let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), hidden_constraints.len(),
                                                             Some(hidden_constraints), Some(observed_constraints));
    constrained_model.train();
    println!("Training time elapsed: {:.2?}", start.elapsed());
    return constrained_model
}

fn generate_sequences(constrained_model: &ConstrainedHiddenMarkov, n: u32) -> Vec<String> {
    let start = Instant::now();
    let mut sequences = vec![];
    for _ in 0..n {
        sequences.push(constrained_model.sample_sequence(true));
    }
    let elapsed = start.elapsed();
    println!("Generation time Elapsed: {:.2?}", elapsed);
    return sequences
}

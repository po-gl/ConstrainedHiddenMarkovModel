use std::fs;
use std::io::Write;
use rand::seq::SliceRandom;
use rand::thread_rng;
use crate::{ConstrainedHiddenMarkov, Constraint, EmptyConstraint, MatchesConstraint, MultiConstraint, StartsWithLetterConstraint};

pub(crate) const START_TOKEN: &str = "<<START>>";

pub(crate) fn get_data(file_path: String) -> String {
    return fs::read_to_string(file_path).expect("Unable to read data file");
}

pub(crate) fn get_test_constraints() -> Vec<Box<dyn Constraint + Send>> {
    // Test Constraints
    let observed_constraints: Vec<Box<dyn Constraint + Send>> = vec![
        Box::new(MultiConstraint::new(vec![
            Box::new(StartsWithLetterConstraint::new('t')),
            Box::new(StartsWithLetterConstraint::new('f')),
        ], false)),
        Box::new(EmptyConstraint::new()),
        Box::new(EmptyConstraint::new()),
        Box::new(MatchesConstraint::new(String::from("red"))),
    ];
    return observed_constraints
}

pub(crate) fn print_sequences(sequences: Vec<String>) {
    for sequence in sequences {
        println!("{}", sequence);
    }
}

pub(crate) fn write_sequences(sequences: Vec<String>, output_file: String) {
    let mut file = fs::File::create(output_file).expect("Unable to create file");
    for seq in sequences {
        file.write_all(seq.as_ref()).expect("Unable to write");
        file.write("\n".as_ref()).expect("Unable to write");
    }
}

pub(crate) fn generate_unique_sequences(constrained_model: &ConstrainedHiddenMarkov, n: i32, out_of: i32) -> Vec<String>{
    // Calculate unique samples out of n samples
    let mut unique = vec![];
    for _ in 0..out_of {
        unique.push(constrained_model.sample_sequence(true));
    }
    unique.sort();
    unique.dedup();
    println!("Unique strings generated: {}/{} = {}", unique.len(), out_of, unique.len() as f32/out_of as f32);

    unique.shuffle(&mut thread_rng());
    let count = if unique.len() < n as usize { unique.len() } else { n as usize };
    return unique[0..count].to_owned()
}
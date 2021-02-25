pub(crate) const START_TOKEN: &str = "<<START>>";

pub(crate) fn print_help() {
    println!("usage: constrained_hmm training_data_file");
    println!("       Note: training sequences in file should be line separated consisting of *observed:hidden* tokens (e.g. Fred:NNP)")
}

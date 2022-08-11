use clap::{Arg, App};
use crate::config::Config;

pub struct Args {
    pub config_file: String,
    pub training_file: String,
    pub constraint_string: String,
    pub markov_order: u32,
    pub num_of_sequences: u32,
    pub output_file: String,
}

impl Args {
    pub fn new() -> Args {
        let matches = App::new("Constrained Hidden Markov Model")
            .about("Generates constrained sequences")
            .arg(Arg::with_name("training_file")
                .short('f')
                .long("file")
                .takes_value(true)
                .help("Training file path"))
            .arg(Arg::with_name("config_file")
                .short('c')
                .long("config")
                .takes_value(true)
                .help("YAML config file path"))
            .arg(Arg::with_name("markov_order")
                .short('m')
                .long("order")
                .takes_value(true)
                .help("Markov order"))
            .arg(Arg::with_name("sequences")
                .short('n')
                .long("sequences")
                .takes_value(true)
                .help("The number of sequences to generate"))
            .arg(Arg::with_name("output_file")
                .short('o')
                .long("out")
                .takes_value(true)
                .help("Output file to write sequences to"))
            .get_matches();

        let config_file = matches.value_of("config_file").unwrap_or("config.yaml").to_string();
        let (training_file, constraint_string, markov_order) = Config::parse(&config_file);

        let args = Args {
            config_file,
            training_file: matches.value_of("training_file").unwrap_or(&training_file).to_string(),
            constraint_string,
            markov_order: matches.value_of("markov_order").unwrap_or(&markov_order).parse::<u32>().unwrap(),
            num_of_sequences: matches.value_of("sequences").unwrap_or("10").parse::<u32>().unwrap(),
            output_file: matches.value_of("output_file").unwrap_or("").to_string(),
        };
        return args
    }
}
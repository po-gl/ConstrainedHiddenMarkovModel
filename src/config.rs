use std::fs;

#[derive(Debug, serde::Deserialize, PartialEq)]
pub struct Config {
    training_file: String,
    markov_order: String,
    constraints: String,
}

impl Config {
    pub fn parse(config_file: &String) -> (String, String, String){
        let config_str = fs::read_to_string(config_file).expect("Unable to read config file");
        let yaml: Config = serde_yaml::from_str(&config_str).unwrap();
        return (yaml.training_file, yaml.constraints, yaml.markov_order)
    }
}
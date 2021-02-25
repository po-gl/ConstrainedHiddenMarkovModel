use std::collections::HashMap;

use rand::Rng;

use crate::utils::START_TOKEN;

#[derive(Debug, Clone)]
pub struct HiddenMarkov {
    pub markov_order: u32,
    pub hidden_probs: HashMap<String, HashMap<String, f64>>,
    pub observed_probs: HashMap<String, HashMap<String, f64>>,
}

impl HiddenMarkov {
    pub fn new(markov_order: u32, data: String) -> HiddenMarkov {
        let mut hmm = HiddenMarkov {
           markov_order,
           hidden_probs: Default::default(),
           observed_probs: Default::default(),
        };

        hmm.train(data);

        return hmm;
    }

    pub fn train(&mut self, data: String) {
        self.clear_probs();

        let lines = data.split("\n");
        for line in lines {
            self.process_line(line)
        }

        self.normalize();
    }

    fn clear_probs(&mut self) {
        self.hidden_probs.clear();
        self.observed_probs.clear();
    }

    fn process_line(&mut self, line: &str) {
        let mut tokens = line.split_whitespace();
        match tokens.next() {
            None => (),
            Some(token) => {
                let mut prev_token: &str = START_TOKEN;
                let mut curr_token: &str = token;
                self.increment(prev_token, curr_token);

                for token in tokens {
                    prev_token = curr_token;
                    curr_token = token;
                    self.increment(prev_token, curr_token);
                }
            }
        }
    }

    fn increment(&mut self, token: &str, next_token: &str) {
        let (_observed, hidden) = HiddenMarkov::split_token(token);
        let (next_observed, next_hidden) = HiddenMarkov::split_token(next_token);

        self.increment_hidden(hidden, next_hidden.to_owned());
        self.increment_observed(next_hidden, next_observed);
    }

    fn increment_hidden(&mut self, hidden: String, next_hidden: String) {
        let inner_hidden_map = self.hidden_probs.entry(hidden).or_insert(HashMap::new());
        inner_hidden_map.entry(next_hidden.to_owned()).or_insert(0.0);
        inner_hidden_map.insert(next_hidden.to_owned(), inner_hidden_map[next_hidden.as_str()] + 1.0);
    }

    fn increment_observed(&mut self, hidden: String, observed: String) {
        let inner_observed_map = self.observed_probs.entry(hidden).or_insert(HashMap::new());
        inner_observed_map.entry(observed.to_owned()).or_insert(0.0);
        inner_observed_map.insert(observed.to_owned(), inner_observed_map[observed.as_str()] + 1.0);
    }

    pub fn split_token(token: &str) -> (String, String) {
        if token.eq(START_TOKEN) { return (String::from(START_TOKEN), String::from(START_TOKEN)) }
        let mut token_split = token.split(":");
        let observed = token_split.next().unwrap();
        let hidden = token_split.next().unwrap();
        (String::from(observed), String::from(hidden))
    }

    fn normalize(&mut self) {
        // One way to save time would be to count sum during increments
        // into separate "normalize_sums" hashmap
        HiddenMarkov::normalize_nested_map(&mut self.hidden_probs);
        HiddenMarkov::normalize_nested_map(&mut self.observed_probs);
    }

    fn normalize_nested_map(map: &mut HashMap<String, HashMap<String, f64>>) {
        for (_, outer_map) in map.iter_mut() {
            let sum: f64 = outer_map.values().sum();
            for (_, inner_map_val) in outer_map.iter_mut() {
                *inner_map_val = *inner_map_val / sum;
            }
        }
    }

    pub fn sample_sequence(&self, length: i32) -> String {
        let mut sequence = String::from("");
        let mut curr_hidden = START_TOKEN;
        for i in 0..length {
            if self.hidden_probs.contains_key(curr_hidden) {
                curr_hidden = HiddenMarkov::next_token(&self.hidden_probs[curr_hidden])
            } else {
                return sequence;
            }

            if self.observed_probs.contains_key(curr_hidden) {
                sequence += HiddenMarkov::next_token(&self.observed_probs[curr_hidden]);
                sequence += ":";
                sequence += curr_hidden;
                if i != length - 1 { sequence += " " }
            }
        }
        return sequence;
    }

    fn next_token(prev_token_map: &HashMap<String, f64>) -> &str {
        let mut sum = 0.0;
        let rand_value: f64 = rand::thread_rng().gen();
        for potential_token in prev_token_map {
            sum += potential_token.1;
            if sum > rand_value {
                return potential_token.0;
            }
        }
        return ""
    }

    pub fn get_sequence_probability(&self, sequence: &str) -> f64 {
        let tokens = sequence.split_whitespace();
        let mut product: f64 = 1.0;
        let mut curr_hidden = String::from(START_TOKEN);

        for token in tokens {
            let (token_observed, token_hidden) = HiddenMarkov::split_token(token);

            product *= self.hidden_probs[&curr_hidden][&token_hidden];
            product *= self.observed_probs[&token_hidden][&token_observed];
            curr_hidden = token_hidden;
        }

        return product;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_hidden_markov() {
        let model = HiddenMarkov {
            markov_order: 1,
            hidden_probs: Default::default(),
            observed_probs: Default::default()
        };

        assert_eq!(1, model.markov_order);
        assert_eq!(HashMap::default(), model.hidden_probs);
        assert_eq!(HashMap::default(), model.observed_probs);
    }

    #[test]
    fn train_hidden_markov() {
        // Data format:
        // * sentences or phrases are line separated
        // * observed and hidden tokens connected by ":" (i.e. *observed*:*hidden*)
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let mut model = HiddenMarkov {
            markov_order: 1,
            hidden_probs: Default::default(),
            observed_probs: Default::default()
        };
        model.train(data);

        assert_eq!(0.4, model.hidden_probs["NNP"]["VBZ"]);
        assert_eq!(0.6, model.hidden_probs["NNP"]["RB"]);
        assert_eq!(0.2, model.observed_probs["NNP"]["Ted"]);
        assert_eq!(0.5, model.observed_probs["VBZ"]["likes"]);
        assert_eq!(0.25, model.observed_probs["VBZ"]["loves"]);
    }

    #[test]
    fn new_hidden_markov() {
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);

        assert_eq!(1, model.markov_order);

        assert_eq!(1.0, model.hidden_probs[START_TOKEN]["NNP"]);
        assert_eq!(0.4, model.hidden_probs["NNP"]["VBZ"]);
        assert_eq!(0.6, model.hidden_probs["NNP"]["RB"]);
        assert_eq!(0.25, model.hidden_probs["VBZ"]["NNP"]);
        assert_eq!(0.75, model.hidden_probs["VBZ"]["NN"]);
        assert_eq!(1.0, model.hidden_probs["RB"]["VBZ"]);

        assert_eq!(0.6, model.observed_probs["NNP"]["Mary"]);
        assert_eq!(0.2, model.observed_probs["NNP"]["Fred"]);
        assert_eq!(0.2, model.observed_probs["NNP"]["Ted"]);
        assert_eq!(0.5, model.observed_probs["VBZ"]["likes"]);
        assert_eq!(0.25, model.observed_probs["VBZ"]["loves"]);
        assert_eq!(0.25, model.observed_probs["VBZ"]["sees"]);
        assert_eq!(1.0/3.0, model.observed_probs["NN"]["green"]);
        assert_eq!(2.0/3.0, model.observed_probs["NN"]["red"]);
        assert_eq!(1.0/3.0, model.observed_probs["RB"]["sometimes"]);
        assert_eq!(2.0/3.0, model.observed_probs["RB"]["now"]);
    }

    #[test]
    fn hidden_markov_generate_sequence() {
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nTed:NNP now:RB likes:VBZ green:NN"
        );
        let model = HiddenMarkov::new(1, data);
        assert_eq!("Ted:NNP now:RB likes:VBZ green:NN", model.sample_sequence(4));
    }

    #[test]
    fn hidden_markov_sequence_probability() {
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        assert_eq!(0.0004999999999999999, model.get_sequence_probability("Ted:NNP sometimes:RB loves:VBZ Fred:NNP"))
    }

    #[test]
    fn clear_hidden_markov() {
        let mut inner_hidden_probs = HashMap::new();
        inner_hidden_probs.insert(String::from("VBZ"), 0.5);
        let mut hidden_probs = HashMap::new();
        hidden_probs.insert(String::from("NNP"), inner_hidden_probs);

        let mut inner_observed_probs = HashMap::new();
        inner_observed_probs.insert(String::from("loves"), 0.25);
        let mut observed_probs = HashMap::new();
        observed_probs.insert(String::from("VBZ"), inner_observed_probs);

        let mut model = HiddenMarkov {
            markov_order: 1,
            hidden_probs,
            observed_probs,
        };

        model.clear_probs();
        assert_eq!(1, model.markov_order);
        assert_eq!(HashMap::default(), model.hidden_probs);
        assert_eq!(HashMap::default(), model.observed_probs);
    }

    #[test]
    fn increment_hidden_prob_hidden_markov() {
        let mut model = HiddenMarkov {
            markov_order: 1,
            hidden_probs: Default::default(),
            observed_probs: Default::default()
        };
        model.increment_hidden(String::from("VBZ"), String::from("NN"));
        model.increment_hidden(String::from("VBZ"), String::from("NN"));

        assert_eq!(2.0, model.hidden_probs["VBZ"]["NN"])
    }

    #[test]
    fn increment_observed_prob_hidden_markov() {
        let mut model = HiddenMarkov {
            markov_order: 1,
            hidden_probs: Default::default(),
            observed_probs: Default::default()
        };
        model.increment_observed(String::from("NN"), String::from("red"));
        model.increment_observed(String::from("NN"), String::from("red"));
        model.increment_observed(String::from("NN"), String::from("red"));

        assert_eq!(3.0, model.observed_probs["NN"]["red"]);
    }

    #[test]
    fn increment_hidden_markov() {
        let mut model = HiddenMarkov {
            markov_order: 1,
            hidden_probs: Default::default(),
            observed_probs: Default::default()
        };
        model.increment(START_TOKEN, START_TOKEN);
        model.increment("loves:VBZ", "red:NN");
        model.increment("loves:VBZ", "red:NN");
        model.increment("sees:VBZ", "green:NN");

        assert_eq!(2.0, model.observed_probs["NN"]["red"]);
        assert_eq!(3.0, model.hidden_probs["VBZ"]["NN"]);
    }

    #[test]
    fn split_token_test() {
        let (observed, hidden) = HiddenMarkov::split_token("Fred:NNP");
        assert_eq!("Fred", observed);
        assert_eq!("NNP", hidden);
    }

    #[test]
    fn split_token_missing() {
        let (observed, hidden) = HiddenMarkov::split_token("Fred:");
        assert_eq!("Fred", observed);
        assert_eq!("", hidden);
    }

    #[test]
    fn normalize_markov() {
        let mut inner_hidden_probs = HashMap::new();
        inner_hidden_probs.insert(String::from("VBZ"), 2.0);
        inner_hidden_probs.insert(String::from("RB"), 3.0);
        let mut hidden_probs = HashMap::new();
        hidden_probs.insert(String::from("NNP"), inner_hidden_probs);

        let mut inner_observed_probs = HashMap::new();
        inner_observed_probs.insert(String::from("likes"), 2.0);
        let mut observed_probs = HashMap::new();
        observed_probs.insert(String::from("VBZ"), inner_observed_probs);

        let mut model = HiddenMarkov {
            markov_order: 1,
            hidden_probs,
            observed_probs,
        };

        assert_eq!(3.0, model.hidden_probs["NNP"]["RB"]);
        assert_eq!(2.0, model.hidden_probs["NNP"]["VBZ"]);
        assert_eq!(2.0, model.observed_probs["VBZ"]["likes"]);
        model.normalize();
        assert_eq!(0.6, model.hidden_probs["NNP"]["RB"]);
        assert_eq!(0.4, model.hidden_probs["NNP"]["VBZ"]);
        assert_eq!(1.0, model.observed_probs["VBZ"]["likes"]);
    }
}
use std::collections::{HashMap, HashSet};
use rand::Rng;
use crate::hidden_markov::HiddenMarkov;
use crate::constraints::Constraint;
use crate::constraints::empty_constraint::EmptyConstraint;
use crate::utils::START_TOKEN;

#[derive(Debug)]
pub struct ConstrainedHiddenMarkov {
    pub hidden_markov_model: HiddenMarkov,
    pub sequence_length: usize,
    pub hidden_probs: Vec<HashMap<String, HashMap<String, f64>>>,
    pub observed_probs: Vec<HashMap<String, HashMap<String, f64>>>,
    pub hidden_constraints: Vec<Box<dyn Constraint + Send>>,
    pub observed_constraints: Vec<Box<dyn Constraint + Send>>,
}

impl ConstrainedHiddenMarkov {
    pub fn new(hidden_markov_model: HiddenMarkov, sequence_length: usize, hidden_constraints: Option<Vec<Box<dyn Constraint + Send>>>, observed_constraints: Option<Vec<Box<dyn Constraint + Send>>>) -> ConstrainedHiddenMarkov {
        let mut chmm = ConstrainedHiddenMarkov {
            hidden_markov_model,
            sequence_length,
            hidden_probs: Default::default(),
            observed_probs: Default::default(),
            hidden_constraints: Default::default(),
            observed_constraints: Default::default()
        };
        assert!(sequence_length > 1);


        chmm.hidden_constraints = hidden_constraints.unwrap_or(
            vec![Box::new(EmptyConstraint::new()); sequence_length]
        );
        chmm.observed_constraints = observed_constraints.unwrap_or(
            vec![Box::new(EmptyConstraint::new()); sequence_length]
        );
        chmm.check_sequence_and_constraint_length();

        return chmm;
    }

    pub fn train(&mut self) {
        self.clear_probs();

        // Copy matrices for each sequence position
        self.duplicate_matrices();

        // Remove states violating the constraints
        self.remove_constrain_violating_states();

        // Enforce arc-consistency
        self.remove_dead_states();

        // Re-normalize
        self.renormalize();
    }

    /// Generate a sequence
    pub fn sample_sequence(&self) -> String {
        let mut sequence = String::from("");
        let mut curr_hidden = START_TOKEN;
        for i in 0..self.sequence_length {
            if self.hidden_probs[i].contains_key(curr_hidden) {
                curr_hidden = ConstrainedHiddenMarkov::next_token(&self.hidden_probs[i][curr_hidden])
            } else {
                return sequence;
            }

            if self.observed_probs[i].contains_key(curr_hidden) {
                sequence += ConstrainedHiddenMarkov::next_token(&self.observed_probs[i][curr_hidden]);
                sequence += ":";
                sequence += curr_hidden;
                if i != self.sequence_length - 1 { sequence += " " }
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

    fn clear_probs(&mut self) {
        self.hidden_probs.clear();
        self.observed_probs.clear();
    }

    fn check_sequence_and_constraint_length(&self) {
        assert_eq!(self.sequence_length, self.hidden_constraints.len());
        assert_eq!(self.sequence_length, self.observed_constraints.len());
    }

    /// Performs deep copy of non-constrained hidden markov model
    /// probabilities for each sequence position
    fn duplicate_matrices(&mut self) {
        for i in 0..self.sequence_length {
            self.hidden_probs.insert(i,self.hidden_markov_model.hidden_probs.clone());
            self.observed_probs.insert(i,self.hidden_markov_model.observed_probs.clone());
        }
    }

    /// Removes states that violate constraints on hidden
    /// or observed sequence positions
    fn remove_constrain_violating_states(&mut self) {
        self.remove_constrain_violating_hidden_states();
        self.remove_constrain_violating_observed_states()
    }

    fn remove_constrain_violating_hidden_states(&mut self) {
        for i in 0..self.hidden_constraints.len() {
            for (_, outer_map) in self.hidden_probs[i].iter_mut() {
                for (inner_map_key, inner_map_val) in outer_map.iter_mut() {
                    // Check for constraint satisfaction
                    if !self.hidden_constraints[i].is_satisfied_by_state(String::from(inner_map_key)) {
                        // TODO: Compare running times of removing probs entirely rather than setting to 0
                        *inner_map_val = 0.0
                    }
                }
            }
        }
    }

    fn remove_constrain_violating_observed_states(&mut self) {
        for i in 0..self.observed_constraints.len() {
            for (_, outer_map) in self.observed_probs[i].iter_mut() {
                for (inner_map_key, inner_map_val) in outer_map.iter_mut() {
                    // Check for constraint satisfaction
                    if !self.observed_constraints[i].is_satisfied_by_state(String::from(inner_map_key)) {
                        *inner_map_val = 0.0;
                    }
                }
            }
        }
    }

    /// Removes state transitions that lead to a zero probability solution
    /// i.e. enforces arc-consistency
    ///
    /// this is a tree-structured CSP, so can be done in a single pass
    fn remove_dead_states(&mut self) {
        // Working backwards through the sequence positions
        // Remove hidden states whose observed state sums to 0.0
        for i in (0..self.hidden_probs.len()).rev() {
            let current_hidden = &mut self.hidden_probs[i];
            let current_observed = &mut self.observed_probs[i];

            // Remove hidden states whose observed state sums to 0.0
            // from the current sequence position
            let states_to_remove_in_curr: HashSet<String> =
                ConstrainedHiddenMarkov::get_zero_sum_outer_keys(current_observed);
            for (_, outer_value) in current_hidden.iter_mut() {
                for (inner_key, inner_value) in outer_value.iter_mut() {
                    if states_to_remove_in_curr.contains(inner_key) {
                        *inner_value = 0.0;
                    }
                }
            }
        }

        // Remove dead states based on transitions
        for i in (1..self.hidden_probs.len()).rev() {
            let current_hidden = &mut self.hidden_probs[i].to_owned();

            // Add states from current sequence position whose transitions sum to 0.0
            // to an array to be removed
            let states_to_remove_in_prev: HashSet<String> =
                ConstrainedHiddenMarkov::get_zero_sum_outer_keys(current_hidden);

            // Remove transitions to removed states in the previous sequence position
            let previous_hidden = &mut self.hidden_probs[i-1];
            for (_, outer_value) in previous_hidden.iter_mut() {
                for (inner_key, inner_value) in outer_value.iter_mut() {
                    if states_to_remove_in_prev.contains(inner_key) {
                        *inner_value = 0.0;
                    }
                    // Also remove transitions to states that do not exist in the next sequence position
                    if current_hidden.get(inner_key).is_none() {
                        *inner_value = 0.0;
                    }
                }
            }
        }
    }

    fn get_zero_sum_outer_keys(probability_matrix: &mut HashMap<String, HashMap<String, f64>>) -> HashSet<String> {
        let mut zero_sum_keys: HashSet<String> = HashSet::new();
        for (outer_key, outer_value) in probability_matrix.iter() {
            if outer_value.values().sum::<f64>() == 0.0 {
                zero_sum_keys.insert(String::from(outer_key));
            }
        }
        return zero_sum_keys;
    }

    /// Re-normalize probabilities such that they have the same
    /// probability distribution as the original HMM
    fn renormalize(&mut self) {

        let mut betas: Vec<HashMap<String, f64>> = vec![HashMap::new(); self.sequence_length];
        let mut alphas: Vec<HashMap<String, f64>> = vec![HashMap::new(); self.sequence_length];

        for i in (0..self.sequence_length).rev() {

            // Renormalize observed values
            for (outer_key, outer_value) in &mut self.observed_probs[i].iter_mut() {
                let sum: f64 = outer_value.values().sum::<f64>();  // beta_j = sum of e_jk
                betas[i].insert(String::from(outer_key), sum);
                if sum != 0.0 {
                    for (_, inner_value) in outer_value.iter_mut() {
                        *inner_value = *inner_value / sum;  // e'_jk = e_jk / beta_j
                    }
                }
            }

            if i == self.sequence_length-1 {
                for (outer_key, outer_value) in &mut self.hidden_probs[i].iter_mut() {
                    let mut sum: f64 = 0.0;
                    for (inner_key, inner_value) in outer_value.iter() {
                        sum += betas[i][inner_key] * inner_value;  // alpha_j = sum of beta_k * z_jk
                    }
                    alphas[i].insert(String::from(outer_key), sum);
                    if sum != 0.0 {
                        for (inner_key, inner_value) in outer_value.iter_mut() {
                            *inner_value = (betas[i][inner_key] * *inner_value) / sum;  // z'_jk = (beta_j * z_jk) / alpha_j
                        }
                    }
                }
            } else {
                for (outer_key, outer_value) in &mut self.hidden_probs[i].iter_mut() {
                    let mut sum: f64 = 0.0;
                    for (inner_key, inner_value) in outer_value.iter() {
                        let alpha: f64;
                        match alphas[i+1].get(inner_key) {
                            Some(value) => alpha = *value,
                            None => alpha = 0.0
                        }
                        sum += betas[i][inner_key] * alpha * inner_value;  // alpha_j = sum of beta_k * alpha^(i+1)_k * z_jk
                    }
                    alphas[i].insert(String::from(outer_key), sum);
                    if sum != 0.0 {
                        for (inner_key, inner_value) in outer_value.iter_mut() {
                            let alpha: f64;
                            match alphas[i+1].get(inner_key) {
                                Some(value) => alpha = *value,
                                None => alpha =0.0
                            }
                            *inner_value = (betas[i][inner_key] * alpha * *inner_value) / sum;  // z'_jk = (beta_j * alpha^(i+1)_k * z_jk) / alpha_j
                        }
                    }
                }
            }
        }
    }

    /// Calculate the probability to generate a given sequence
    pub fn get_sequence_probability(&self, sequence: &str) -> f64 {
        let tokens = sequence.split_whitespace();
        let mut product: f64 = 1.0;
        let mut curr_hidden = String::from(START_TOKEN);

        let mut i: usize = 0;
        for token in tokens {
            let (token_observed, token_hidden) = HiddenMarkov::split_token(token);
            product *= self.hidden_probs[i][&curr_hidden][&token_hidden];
            product *= self.observed_probs[i][&token_hidden][&token_observed];
            curr_hidden = token_hidden;
            i += 1;
        }
        return product;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::empty_constraint::EmptyConstraint;
    use crate::constraints::starts_with_letter_constraint::StartsWithLetterConstraint;
    use crate::constraints::matches_constraint::MatchesConstraint;
    use crate::utils::START_TOKEN;
    use crate::constraints::multi_constraint::MultiConstraint;

    #[test]
    fn create_constrained_hidden_markov() {
        let model = ConstrainedHiddenMarkov {
            hidden_markov_model: HiddenMarkov::new(1, Default::default()),
            sequence_length: 4,
            hidden_probs: Default::default(),
            observed_probs: Default::default(),
            hidden_constraints: Default::default(),
            observed_constraints: Default::default()
        };
        assert_eq!(1, model.hidden_markov_model.markov_order);
        assert_eq!(4, model.sequence_length);
    }

    #[test]
    fn clear_probabilities_chmm() {
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, None);

        constrained_model.hidden_probs.push(model.hidden_probs.clone());
        constrained_model.hidden_probs.push(model.hidden_probs.clone());
        constrained_model.hidden_probs.push(model.hidden_probs.clone());
        constrained_model.observed_probs.push(model.observed_probs.clone());
        constrained_model.observed_probs.push(model.observed_probs.clone());

        assert_eq!(3, constrained_model.hidden_probs.len());
        assert_eq!(2, constrained_model.observed_probs.len());

        constrained_model.clear_probs();
        assert_eq!(0, constrained_model.hidden_probs.len());
        assert_eq!(0, constrained_model.observed_probs.len());
    }
    
    #[test]
    fn constraints_chmm() {
        let constraints: Vec<Box<dyn Constraint + Send>> = vec![
            Box::new(StartsWithLetterConstraint::new('f')),
            Box::new(EmptyConstraint::new()),
            Box::new(EmptyConstraint::new()),
            Box::new(MatchesConstraint::new(String::from("red"))),
        ];
        let model = ConstrainedHiddenMarkov {
            hidden_markov_model: HiddenMarkov::new(1, Default::default()),
            sequence_length: 4,
            hidden_probs: Default::default(),
            observed_probs: Default::default(),
            hidden_constraints: Default::default(),
            observed_constraints: constraints
        };
        assert_eq!(4, model.observed_constraints.len());
        assert_eq!(true, model.observed_constraints[0].is_satisfied_by_state(String::from("Fred")));
        assert_eq!(true, model.observed_constraints[1].is_satisfied_by_state(String::from("anything")));
        assert_eq!(true, model.observed_constraints[2].is_satisfied_by_state(String::from("")));
        assert_eq!(false, model.observed_constraints[3].is_satisfied_by_state(String::from("reds")));
    }

    #[test]
    fn remove_constraint_violating_states_chmm() {
        let observed_constraints: Vec<Box<dyn Constraint + Send>> = vec![
            Box::new(StartsWithLetterConstraint::new('t')),
            Box::new(EmptyConstraint::new()),
            Box::new(EmptyConstraint::new()),
            Box::new(MatchesConstraint::new(String::from("red"))),
        ];
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, Some(observed_constraints));

        constrained_model.duplicate_matrices();
        constrained_model.remove_constrain_violating_states();
        assert_eq!(0.0, constrained_model.observed_probs[0]["VBZ"]["likes"]);
        assert_eq!(0.2, constrained_model.observed_probs[0]["NNP"]["Ted"]);

        assert_eq!(0.5, constrained_model.observed_probs[1]["VBZ"]["likes"]);
        assert_eq!(0.2, constrained_model.observed_probs[1]["NNP"]["Ted"]);

        assert_eq!(2.0/3.0, constrained_model.observed_probs[3]["NN"]["red"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["NN"]["green"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["RB"]["now"]);
    }

    #[test]
    fn remove_dead_nodes_from_hidden_constraints() {
        let hidden_constraints: Vec<Box<dyn Constraint + Send>> = vec![
            Box::new(EmptyConstraint::new()),
            Box::new(EmptyConstraint::new()),
            Box::new(EmptyConstraint::new()),
            Box::new(MatchesConstraint::new(String::from("NNP"))),
        ];
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, Some(hidden_constraints), None);
        constrained_model.duplicate_matrices();
        constrained_model.remove_constrain_violating_states();

        constrained_model.remove_dead_states();

        assert_eq!(0.25, constrained_model.hidden_probs[3]["VBZ"]["NNP"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3]["VBZ"]["NN"]);

        assert_eq!(0.4, constrained_model.hidden_probs[2]["NNP"]["VBZ"]);
        assert_eq!(1.0, constrained_model.hidden_probs[2]["RB"]["VBZ"]);
        assert_eq!(0.0, constrained_model.hidden_probs[2]["NNP"]["RB"]);
        assert_eq!(0.0, constrained_model.hidden_probs[2]["VBZ"]["NN"]);
        assert_eq!(0.0, constrained_model.hidden_probs[2]["VBZ"]["NNP"]);

        assert_eq!(0.6, constrained_model.hidden_probs[1]["NNP"]["RB"]);
        assert_eq!(0.25, constrained_model.hidden_probs[1]["VBZ"]["NNP"]);
        assert_eq!(0.0, constrained_model.hidden_probs[1]["RB"]["VBZ"]);
    }

    #[test]
    fn remove_dead_nodes_from_observed_constraints() {
        let observed_constraints: Vec<Box<dyn Constraint + Send>> = vec![
            Box::new(StartsWithLetterConstraint::new('t')),
            Box::new(EmptyConstraint::new()),
            Box::new(EmptyConstraint::new()),
            Box::new(MatchesConstraint::new(String::from("red"))),
        ];
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, Some(observed_constraints));
        constrained_model.duplicate_matrices();
        constrained_model.remove_constrain_violating_states();

        constrained_model.remove_dead_states();

        assert_eq!(1.0, constrained_model.hidden_probs[0][START_TOKEN]["NNP"]);
        assert_eq!(0.25, constrained_model.hidden_probs[0]["VBZ"]["NNP"]);
        assert_eq!(0.0, constrained_model.hidden_probs[0]["NNP"]["RB"]);
        assert_eq!(0.0, constrained_model.hidden_probs[0]["NNP"]["VBZ"]);
        assert_eq!(0.0, constrained_model.hidden_probs[0]["RB"]["VBZ"]);
        assert_eq!(0.0, constrained_model.hidden_probs[0]["VBZ"]["NN"]);

        assert_eq!(0.4, constrained_model.hidden_probs[2]["NNP"]["VBZ"]);
        assert_eq!(0.0, constrained_model.hidden_probs[2]["NNP"]["RB"]);

        assert_eq!(0.75, constrained_model.hidden_probs[3]["VBZ"]["NN"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3]["NNP"]["RB"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3]["RB"]["VBZ"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3]["NNP"]["VBZ"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3]["VBZ"]["NNP"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3][START_TOKEN]["NNP"]);
    }

    #[test]
    fn renormalize_chmm() {
        let observed_constraints: Vec<Box<dyn Constraint + Send>> = vec![
            Box::new(MultiConstraint::new(vec![
                Box::new(StartsWithLetterConstraint::new('t')),
                Box::new(StartsWithLetterConstraint::new('f')),
            ], false)),
            Box::new(EmptyConstraint::new()),
            Box::new(EmptyConstraint::new()),
            Box::new(MatchesConstraint::new(String::from("red"))),
        ];
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, Some(observed_constraints));
        constrained_model.duplicate_matrices();
        constrained_model.remove_constrain_violating_states();
        constrained_model.remove_dead_states();

        constrained_model.renormalize();

        assert_eq!(1.0, constrained_model.hidden_probs[0]["VBZ"]["NNP"]);
        assert_eq!(1.0, constrained_model.hidden_probs[1]["NNP"]["RB"]);
        assert_eq!(1.0, constrained_model.hidden_probs[1]["VBZ"]["NNP"]);
        assert_eq!(0.0, constrained_model.hidden_probs[1]["VBZ"]["NN"]);
        assert_eq!(1.0, constrained_model.hidden_probs[2]["NNP"]["VBZ"]);
        assert_eq!(0.0, constrained_model.hidden_probs[2]["NNP"]["RB"]);
        assert_eq!(1.0, constrained_model.hidden_probs[2]["RB"]["VBZ"]);
        assert_eq!(1.0, constrained_model.hidden_probs[3]["VBZ"]["NN"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3]["VBZ"]["NNP"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3]["RB"]["VBZ"]);

        assert_eq!(0.5, constrained_model.observed_probs[0]["NNP"]["Fred"]);
        assert_eq!(0.5, constrained_model.observed_probs[0]["NNP"]["Ted"]);
        assert_eq!(0.0, constrained_model.observed_probs[0]["NNP"]["Mary"]);
        assert_eq!(0.0, constrained_model.observed_probs[0]["VBZ"]["likes"]);
        assert_eq!(0.25, constrained_model.observed_probs[1]["VBZ"]["sees"]);
        assert_eq!(0.25, constrained_model.observed_probs[1]["VBZ"]["loves"]);
        assert_eq!(0.5, constrained_model.observed_probs[1]["VBZ"]["likes"]);
        assert_eq!(1.0/3.0, constrained_model.observed_probs[1]["NN"]["green"]);
        assert_eq!(2.0/3.0, constrained_model.observed_probs[1]["NN"]["red"]);
        assert_eq!(0.2, constrained_model.observed_probs[1]["NNP"]["Fred"]);
        assert_eq!(0.6, constrained_model.observed_probs[1]["NNP"]["Mary"]);
        assert_eq!(0.2, constrained_model.observed_probs[1]["NNP"]["Ted"]);
        assert_eq!(0.2, constrained_model.observed_probs[2]["NNP"]["Fred"]);
        assert_eq!(0.6, constrained_model.observed_probs[2]["NNP"]["Mary"]);
        assert_eq!(0.2, constrained_model.observed_probs[2]["NNP"]["Ted"]);
        assert_eq!(1.0, constrained_model.observed_probs[3]["NN"]["red"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["NN"]["green"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["NNP"]["Fred"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["NNP"]["Mary"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["NNP"]["Ted"]);
    }

    #[test]
    fn train_chmm() {
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
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, Some(observed_constraints));
        constrained_model.train();

        assert_eq!(1.0, constrained_model.hidden_probs[0]["VBZ"]["NNP"]);
        assert_eq!(1.0, constrained_model.hidden_probs[1]["NNP"]["RB"]);
        assert_eq!(1.0, constrained_model.hidden_probs[1]["VBZ"]["NNP"]);
        assert_eq!(0.0, constrained_model.hidden_probs[1]["VBZ"]["NN"]);
        assert_eq!(1.0, constrained_model.hidden_probs[2]["NNP"]["VBZ"]);
        assert_eq!(0.0, constrained_model.hidden_probs[2]["NNP"]["RB"]);
        assert_eq!(1.0, constrained_model.hidden_probs[2]["RB"]["VBZ"]);
        assert_eq!(1.0, constrained_model.hidden_probs[3]["VBZ"]["NN"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3]["VBZ"]["NNP"]);
        assert_eq!(0.0, constrained_model.hidden_probs[3]["RB"]["VBZ"]);

        assert_eq!(0.5, constrained_model.observed_probs[0]["NNP"]["Fred"]);
        assert_eq!(0.5, constrained_model.observed_probs[0]["NNP"]["Ted"]);
        assert_eq!(0.0, constrained_model.observed_probs[0]["NNP"]["Mary"]);
        assert_eq!(0.0, constrained_model.observed_probs[0]["VBZ"]["likes"]);
        assert_eq!(0.25, constrained_model.observed_probs[1]["VBZ"]["sees"]);
        assert_eq!(0.25, constrained_model.observed_probs[1]["VBZ"]["loves"]);
        assert_eq!(0.5, constrained_model.observed_probs[1]["VBZ"]["likes"]);
        assert_eq!(1.0/3.0, constrained_model.observed_probs[1]["NN"]["green"]);
        assert_eq!(2.0/3.0, constrained_model.observed_probs[1]["NN"]["red"]);
        assert_eq!(0.2, constrained_model.observed_probs[1]["NNP"]["Fred"]);
        assert_eq!(0.6, constrained_model.observed_probs[1]["NNP"]["Mary"]);
        assert_eq!(0.2, constrained_model.observed_probs[1]["NNP"]["Ted"]);
        assert_eq!(0.2, constrained_model.observed_probs[2]["NNP"]["Fred"]);
        assert_eq!(0.6, constrained_model.observed_probs[2]["NNP"]["Mary"]);
        assert_eq!(0.2, constrained_model.observed_probs[2]["NNP"]["Ted"]);
        assert_eq!(1.0, constrained_model.observed_probs[3]["NN"]["red"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["NN"]["green"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["NNP"]["Fred"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["NNP"]["Mary"]);
        assert_eq!(0.0, constrained_model.observed_probs[3]["NNP"]["Ted"]);
    }

    #[test]
    fn generate_sequence_chmm() {
        let observed_constraints: Vec<Box<dyn Constraint + Send>> = vec![
            Box::new(MultiConstraint::new(vec![
                Box::new(StartsWithLetterConstraint::new('t')),
                Box::new(StartsWithLetterConstraint::new('f')),
            ], false)),
            Box::new(EmptyConstraint::new()),
            Box::new(EmptyConstraint::new()),
            Box::new(MatchesConstraint::new(String::from("green"))),
        ];
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nTed:NNP now:RB likes:VBZ green:NN"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, Some(observed_constraints));
        constrained_model.train();
        assert_eq!("Ted:NNP now:RB likes:VBZ green:NN", constrained_model.sample_sequence());
    }

    #[test]
    fn generate_random_sequence_chmm() {
        let observed_constraints: Vec<Box<dyn Constraint + Send>> = vec![
            Box::new(MultiConstraint::new(vec![
                Box::new(StartsWithLetterConstraint::new('t')),
                Box::new(StartsWithLetterConstraint::new('f')),
            ], false)),
            Box::new(EmptyConstraint::new()),
            Box::new(EmptyConstraint::new()),
            Box::new(MatchesConstraint::new(String::from("red"))),
        ];
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, Some(observed_constraints));
        constrained_model.train();
        assert_eq!(true, constrained_model.sample_sequence().ends_with("red:NN"));
    }

    #[test]
    fn sequence_probability_chmm() {
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, None);
        constrained_model.train();
        assert_eq!(0.0007142857142857144, constrained_model.get_sequence_probability("Ted:NNP sometimes:RB loves:VBZ Fred:NNP"))
    }

    #[test]
    fn sequence_probability_two_chmm() {
        let observed_constraints: Vec<Box<dyn Constraint + Send>> = vec![
            Box::new(MultiConstraint::new(vec![
                Box::new(StartsWithLetterConstraint::new('t')),
                Box::new(StartsWithLetterConstraint::new('f')),
            ], false)),
            Box::new(EmptyConstraint::new()),
            Box::new(EmptyConstraint::new()),
            Box::new(MatchesConstraint::new(String::from("red"))),
        ];
        let data = String::from(
            "Ted:NNP now:RB likes:VBZ green:NN\nMary:NNP likes:VBZ red:NN\nMary:NNP now:RB loves:VBZ red:NN\nFred:NNP sees:VBZ Mary:NNP sometimes:RB"
        );
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 4, None, Some(observed_constraints));
        constrained_model.train();
        assert_eq!(1.0/6.0, constrained_model.get_sequence_probability("Ted:NNP now:RB likes:VBZ red:NN"));
        assert_eq!(1.0/12.0, constrained_model.get_sequence_probability("Ted:NNP now:RB loves:VBZ red:NN"));
        assert_eq!(1.0/12.0, constrained_model.get_sequence_probability("Ted:NNP now:RB sees:VBZ red:NN"));
        assert_eq!(1.0/12.0, constrained_model.get_sequence_probability("Ted:NNP sometimes:RB likes:VBZ red:NN"));
        assert_eq!(1.0/24.0, constrained_model.get_sequence_probability("Ted:NNP sometimes:RB loves:VBZ red:NN"));
        assert_eq!(1.0/24.0, constrained_model.get_sequence_probability("Ted:NNP sometimes:RB sees:VBZ red:NN"));
        assert_eq!(1.0/6.0, constrained_model.get_sequence_probability("Fred:NNP now:RB likes:VBZ red:NN"));
        assert_eq!(1.0/12.0, constrained_model.get_sequence_probability("Fred:NNP now:RB loves:VBZ red:NN"));
        assert_eq!(1.0/12.0, constrained_model.get_sequence_probability("Fred:NNP now:RB sees:VBZ red:NN"));
        assert_eq!(1.0/12.0, constrained_model.get_sequence_probability("Fred:NNP sometimes:RB likes:VBZ red:NN"));
        assert_eq!(1.0/24.0, constrained_model.get_sequence_probability("Fred:NNP sometimes:RB loves:VBZ red:NN"));
        assert_eq!(1.0/24.0, constrained_model.get_sequence_probability("Fred:NNP sometimes:RB sees:VBZ red:NN"));
    }
}
use std::fs;
use std::time::{Duration, Instant};
use crate::{ConstrainedHiddenMarkov, HiddenMarkov};

fn time_analysis_alphabet_size() {
    // Make sure the strings are equal length even if there aren't a lot of unique
    // tokens, they should still be like 000000001 or something
    //
    let avg_count = 5;
    let avg_gen_count = 10;
    let mut data_str:String = String::from("alphabet_size,average_train_time,average_gen_time\n");

    for alphabet_size in 5..=100 {
        let mut avg_train_sum = Duration::new(0, 0);
        let mut avg_gen_sum = Duration::new(0, 0);
        for _ in 0..avg_count {

            // Make arbitrary database
            let mut data:String = "".to_string();
            for i in 0..alphabet_size {
                for j in 0..alphabet_size {
                    data.push_str(&*format!("{:04}:{:04} ", i, j));
                }
                data.push('\n');
            }
            // println!("DATA:\n{}", data);

            // Time Markov model training
            let start = Instant::now();
            let model = HiddenMarkov::new(2, data);
            let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 10,
                                                                     None, None);
            constrained_model.train();
            avg_train_sum += start.elapsed();

        }

        // Make arbitrary database
        let mut data:String = "".to_string();
        for i in 0..alphabet_size {
            for j in 0..alphabet_size {
                data.push_str(&*format!("{:04}:{:04} ", i, j));
            }
            data.push('\n');
        }
        data.push_str(&*format!("{:04}:{:04} ", 0, alphabet_size-1));
        data.push_str(&*format!("{:04}:{:04} ", 0, 0));
        data.push('\n');

        // Time Markov model training
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), 10,
                                                                 None, None);
        constrained_model.train();

        for _ in 0..avg_gen_count {
            let start = Instant::now();
            let _str = constrained_model.sample_sequence(false);
            avg_gen_sum += start.elapsed();
            // println!("Gen: {}", _str);
        }

        println!("Alphabet size: {}  total elapsed time: {:.3?}   average train time: {:.3?} average gen time: {:.3?}", alphabet_size, avg_train_sum, avg_train_sum /avg_count, avg_gen_sum/avg_gen_count);
        data_str.push_str(&*format!("{},{:.3?},{:.3?}\n", alphabet_size, avg_train_sum / avg_count, avg_gen_sum / avg_gen_count));
    }
    fs::write("MarkovRunningTimes.csv", data_str).expect("Unable to write to file.");
}

fn time_analysis_seq_length() {
    let avg_count = 2;
    let avg_gen_count = 3;
    let alphabet_size = 10;

    let mut data_str:String = String::from("sequence_length,average_train_time,average_gen_time\n");

    for seq_length in (50..=10000).step_by(50) {
        // for seq_length in (15..=25).step_by(5) {
        let mut avg_train_sum = Duration::new(0, 0);
        let mut avg_gen_sum = Duration::new(0, 0);

        for _ in 0..avg_count {

            // Make arbitrary database
            let mut data:String = "".to_string();
            for i in 0..alphabet_size {
                for j in 0..alphabet_size {
                    data.push_str(&*format!("{:04}:{:04} ", i, j));
                }
                data.push('\n');
            }
            // Time Markov model training
            let start = Instant::now();
            let model = HiddenMarkov::new(2, data);
            let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), seq_length,
                                                                     None, None);
            constrained_model.train();
            avg_train_sum += start.elapsed();
        }

        // Make arbitrary database
        let mut data:String = "".to_string();
        for i in 0..alphabet_size {
            for j in 0..alphabet_size {
                data.push_str(&*format!("{:04}:{:04} ", i, j));
            }
            data.push('\n');
        }
        data.push_str(&*format!("{:04}:{:04} ", 0, alphabet_size-1));
        data.push_str(&*format!("{:04}:{:04} ", 0, 0));
        data.push('\n');

        // Time Markov model training
        let model = HiddenMarkov::new(1, data);
        let mut constrained_model = ConstrainedHiddenMarkov::new(model.clone(), seq_length,
                                                                 None, None);
        constrained_model.train();

        for _ in 0..avg_gen_count {
            let start = Instant::now();
            let _str = constrained_model.sample_sequence(false);
            avg_gen_sum += start.elapsed();
        }
        println!("Sequence length: {}  total elapsed time: {:.3?}   average train time: {:.3?} average gen time: {:.3?}", seq_length, avg_train_sum, avg_train_sum /avg_count, avg_gen_sum/avg_gen_count);
        data_str.push_str(&*format!("{},{:.3?},{:.3?}\n", seq_length, avg_train_sum / avg_count, avg_gen_sum / avg_gen_count));
    }
    fs::write("MarkovRunningTimesLengths.csv", data_str).expect("Unable to write to file.");
}

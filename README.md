# ConstrainedHiddenMarkovModel

This novel model was implemented as part of my [Master's thesis](https://porterglines.com/assets/Glines_Porter_MS.pdf) to generate musical sequences
styled after Bach chorales (also see https://github.com/po-gl/BachPipeline).
The constrained hidden Markov processes is a novel extension of work done 
by Pachet et al. in their paper, *"Finite-Length Markov Processes with Constraints"*.

The model generates sequences and can apply user-defined constraints to the sequences. 
Sequences could be generated in any number of domains such as natural language or music generation.

<img width="800" alt="trained model for toy example" src="https://user-images.githubusercontent.com/42399205/186280300-829557c8-fa82-48a8-8509-d9b772e412e4.png">

Above is a visual representation of a trained constrained hidden Markov model.

## Usage
Ensure that ```cargo``` is installed then run using the following command inside the project directory.
```
cargo run -- -n 10 -c config.yaml
```

Available options are listed below:

```
USAGE:
    constrained_hmm [OPTIONS]

OPTIONS:
    -c, --config <config_file>     YAML config file path
    -f, --file <training_file>     Training file path
    -h, --help                     Print help information
    -m, --order <markov_order>     Markov order
    -n, --sequences <sequences>    The number of sequences to generate
    -o, --out <output_file>        Output file to write sequences to
```

Constraints are specified by the YAML config file. See ```config.yaml``` for an example.

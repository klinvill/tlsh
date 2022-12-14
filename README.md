# TLSH Implementation and Evaluation

This repo contains an implementation of the Trend Micro Locality Sensitive Hash (TLSH) and an evaluation of TLSH
against ssdeep, another fuzzy hash. This implementation and comparison was done for a class mini-project.

## Running the code
To run this project, you first need to install Rust and Cargo. The recommended way to install Rust is through rustup
(https://rustup.rs/). If you also want to generate the plots from the evaluation results, you will also need to install python
and the packages mentioned in `evaluation/analysis/requirements.txt`. You can install the required packages by running the command
`pip install -r evaluation/analysis/requirements.txt` from this directory. I highly recommend using a conda environment from the
anaconda project (https://anaconda.org/).

### Using TLSH
To use the TLSH implementation, you can build it as a library and then link it in another crate using the approach
described at https://doc.rust-lang.org/rust-by-example/crates/lib.html. The library exposes two functions: `hash()`
generates a TLSH hash of the provided data, and `diff()` computes the distance between two TLSH hashes.

### Running Tests
You can run the tests by running `cargo test` from this directory.

### Fuzzing the Implementation
You can fuzz the TLSH implementation against a reference TLSH binary. To do this, simply set the TLSH_BIN environment
variable to point to the reference binary, and run the command `cargo fuzz run reference_tlsh`. 

### Generating Evaluation Plots
You can generate the plots by running `python analysis.py` from the `evaluation/analysis` directory. This script
expects the relevant evaluation result CSVs to already exist.

### Generating Evaluation Results
Evaluation results from my runs are already present in the repo under `evaluation/results`. However, if you want to
generate your own results, you can simply run `cargo run` from the `evaluation` directory. 

### Generating Malware Payloads
The malware payloads are generated by calling an existing Metasploit executable. If you have msfvenom installed on your
system and in your path, simply run `python gen_payloads.py` from the `evaluation/metasploit` directory to generate the
base and encoded payloads.

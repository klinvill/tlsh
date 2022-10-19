use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use csv::WriterBuilder;
use tlsh;
use ssdeep;

mod alter_files;

// Runs experiment where the small permutations are performed on the first 500 lines of Pride and
// Prejudice
fn small_experiment() {
    let iterations = 500;

    let mut infile = File::open("source_files/pg1342.txt").unwrap();
    let mut buf = String::new();
    infile.read_to_string(&mut buf).unwrap();
    let first_500 = buf.lines().take(500).collect::<Vec<_>>().join("\n");

    let mut text = alter_files::AlteredText::new(first_500.chars().collect());
    let mut outfile = WriterBuilder::new().from_path("results/pg1342_500.txt").unwrap();
    outfile.write_record(&["Iteration", "TLSH Diff", "ssdeep Similarity"]).unwrap();

    fn chars_to_bytes(chars: &[char]) -> Vec<u8> {
        chars.iter().collect::<String>().bytes().collect()
    }

    let base_tlsh = tlsh::hash(&chars_to_bytes(text.text())).unwrap();
    let base_ssdeep = ssdeep::hash(&chars_to_bytes(text.text())).unwrap();

    for i in 0..iterations {
        // Only permute once then check the hash similarity scores
        text.small_permute(1);
        let bytes = chars_to_bytes(text.text());
        let new_tlsh = tlsh::hash(&bytes).unwrap();
        let new_ssdeep = ssdeep::hash(&bytes).unwrap();

        let tlsh_diff = tlsh::diff(&base_tlsh, &new_tlsh).unwrap();
        let ssdeep_diff = ssdeep::compare(base_ssdeep.as_bytes(), new_ssdeep.as_bytes()).unwrap();

        println!("Iteration: {i}, TLSH diff: {tlsh_diff}, ssdeep diff: {ssdeep_diff}");
        outfile.write_record(&[i.to_string(), tlsh_diff.to_string(), ssdeep_diff.to_string()]).unwrap();
    }
}

// Runs experiment where the large permutations are performed on the full Pride and Prejudice text
fn large_experiment() {
    let iterations = 500;

    let mut infile = File::open("source_files/pg1342.txt").unwrap();
    let mut buf = String::new();
    infile.read_to_string(&mut buf).unwrap();

    let mut text = alter_files::AlteredText::new(buf.chars().collect());
    let mut outfile = WriterBuilder::new().from_path("results/pg1342.txt").unwrap();
    outfile.write_record(&["Iteration", "TLSH Diff", "ssdeep Similarity"]).unwrap();

    fn chars_to_bytes(chars: &[char]) -> Vec<u8> {
        chars.iter().collect::<String>().bytes().collect()
    }

    let base_tlsh = tlsh::hash(&chars_to_bytes(text.text())).unwrap();
    let base_ssdeep = ssdeep::hash(&chars_to_bytes(text.text())).unwrap();

    for i in 0..iterations {
        // Only permute once then check the hash similarity scores
        text.large_permute(1);
        let bytes = chars_to_bytes(text.text());
        let new_tlsh = tlsh::hash(&bytes).unwrap();
        let new_ssdeep = ssdeep::hash(&bytes).unwrap();

        let tlsh_diff = tlsh::diff(&base_tlsh, &new_tlsh).unwrap();
        let ssdeep_diff = ssdeep::compare(base_ssdeep.as_bytes(), new_ssdeep.as_bytes()).unwrap();

        println!("Iteration: {i}, TLSH diff: {tlsh_diff}, ssdeep diff: {ssdeep_diff}");
        outfile.write_record(&[i.to_string(), tlsh_diff.to_string(), ssdeep_diff.to_string()]).unwrap();
    }
}

fn main() {
    small_experiment();
    large_experiment();
}

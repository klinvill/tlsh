use std::path::Path;
use std::ptr::write;
use csv;
use itertools::Itertools;
use ssdeep;
use tlsh;

struct Entry {
    file: String,
    is_distinct: bool,
    base_file: String,
    tlsh_hash: String,
    ssdeep_hash: String,
}

struct ComparisonEntry {
    file1: String,
    file2: String,
    are_distinct: bool,
    tlsh_diff: i32,
    ssdeep_similarity: i32,
}

impl Entry {
    fn new_distinct(file: &str, tlsh_hash: &str, ssdeep_hash: &str) -> Self {
        Entry {
            file: file.to_string(),
            is_distinct: true,
            base_file: file.to_string(),
            tlsh_hash: tlsh_hash.to_string(),
            ssdeep_hash: ssdeep_hash.to_string(),
        }
    }

    fn new_similar(file: &str, base_file: &str, tlsh_hash: &str, ssdeep_hash: &str) -> Self {
        Entry {
            file: file.to_string(),
            is_distinct: false,
            base_file: base_file.to_string(),
            tlsh_hash: tlsh_hash.to_string(),
            ssdeep_hash: ssdeep_hash.to_string(),
        }
    }
}

fn get_linux_bin_entries() -> Vec<Entry> {
    // The linux bins csv file was hand-curated to add information on similar files (from the
    // distinct hashes csv). This was needed because some bins (like 7z, 7za, and 7zr) were very
    // similar.
    let linux_bins_csv = Path::new("results/linux_bins.csv");
    let mut reader = csv::Reader::from_path(linux_bins_csv).unwrap();

    // Ignore the CSV header
    reader.records().skip(1)
        .map(|record| {
            let _record = record.unwrap();
            // The second column in the Linux Bins file only contains an entry if it is similar to another file.
            if _record[1].len() == 0 {
                Entry::new_distinct(&_record[0], &_record[2], &_record[3])
            } else {
                Entry::new_similar(&_record[0], &_record[1], &_record[2], &_record[3])
            }
        })
        .collect()
}

fn get_malware_entries() -> Vec<Entry> {
    let malware_csv = Path::new("results/similar_hashes.csv");
    let mut reader = csv::Reader::from_path(malware_csv).unwrap();

    // Ignore the CSV header
    reader.records().skip(1)
        .map(|record| {
            let _record = record.unwrap();
            // All malware entries have a similar file since there are multiple encodings of the
            // same base payload for each payload
            Entry::new_similar(&_record[0], &_record[3], &_record[1], &_record[2])
        })
        .collect()
}

fn comparisons() -> Vec<ComparisonEntry> {
    let linux_bin_entries = get_linux_bin_entries();
    let malware_entries = get_malware_entries();

    // Roughly n choose 2 results
    let mut results = Vec::with_capacity(
        linux_bin_entries.len() * linux_bin_entries.len() / 2
        + malware_entries.len() * malware_entries.len() / 2
    );

    for entries in [&linux_bin_entries, &malware_entries] {
        // Don't use the last entry as e1 since it will need to be used as e2
        for (i, e1) in entries.iter().enumerate().take(entries.len()-1) {
            for e2 in entries[i+1..].iter() {
                let tlsh_diff = tlsh::diff(&e1.tlsh_hash, &e2.tlsh_hash).unwrap();
                let ssdeep_similarity = ssdeep::compare(&e1.ssdeep_hash.as_bytes(), &e2.ssdeep_hash.as_bytes()).unwrap() as i32;
                results.push(ComparisonEntry {
                    file1: e1.file.to_string(),
                    file2: e2.file.to_string(),
                    are_distinct: e1.base_file != e2.base_file,
                    tlsh_diff,
                    ssdeep_similarity,
                });
            }
        }
    }

    results
}

pub(crate) fn comparison_experiment() {
    let results_file = Path::new("results/comparisons.csv");
    let comparison_results = comparisons();

    let mut writer = csv::Writer::from_path(results_file).unwrap();
    writer.write_record(["File 1", "File 2", "Are Distinct", "TLSH Diff", "ssdeep Similarity"]).unwrap();
    for entry in comparison_results {
        writer.write_record([
            entry.file1.to_string(),
            entry.file2.to_string(),
            entry.are_distinct.to_string(),
            entry.tlsh_diff.to_string(),
            entry.ssdeep_similarity.to_string()
        ]).unwrap();
    }
}

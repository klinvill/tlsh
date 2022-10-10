#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate tlsh;
use std::process;


fuzz_target!(|data: &[u8]| {
    // The TLSH reference binary requires a file to hash
    let mut temp_file = std::env::temp_dir();
    temp_file.push("tlsh_fuzzing_temp.bin");
    std::fs::write(temp_file.clone(), data).unwrap();

    let tlsh_bin = std::env::var("TLSH_BIN")
        .expect("Must set the TLSH_BIN environment variable to the reference TLSH executable.");

    let reference_result = process::Command::new(tlsh_bin)
            .args(["-f", temp_file.to_str().unwrap()])
            .output()
            .expect("Failed to execute reference TLSH binary");

    let reference_out = String::from_utf8(reference_result.stdout).unwrap();

    // The reference binary spits out the hash and then the filename, separated by a tab
    let expected_hash = reference_out.split('\t').next().unwrap();

    if !reference_result.status.success() || expected_hash == "" {
        // Only fuzz inputs that don't error out or return nothing for the reference should be used.
        return;
    }

    let my_hash = tlsh::hash(data);
    assert_eq!(my_hash.unwrap(), expected_hash);
});

use std::fs::OpenOptions;
use std::path::Path;
use indicatif::ProgressBar;
use tlsh;
use ssdeep;

pub(crate) fn collect_hashes(source_dir: &Path, results_csv: &Path) {
    let write_header = !results_csv.exists();

    let outfile = OpenOptions::new()
        .create(true)
        .append(true)
        .open(results_csv)
        .unwrap();
    let mut writer = csv::Writer::from_writer(outfile);

    if write_header {
        writer.write_record(["File", "TLSH Hash", "ssdeep Hash"]).unwrap();
    }

    let num_entries = source_dir.read_dir().unwrap().count();
    let prog = ProgressBar::new(num_entries.try_into().unwrap());

    for entry in prog.wrap_iter(source_dir.read_dir().unwrap()) {
        let _entry = entry.unwrap();
        if _entry.file_type().unwrap().is_file() {
            let try_data = std::fs::read(_entry.path());
            if let Ok(data) = try_data {
                let tlsh_hash = tlsh::hash(&data).unwrap();
                let ssdeep_hash = ssdeep::hash(&data).unwrap();
                writer.write_record([_entry.path().to_str().unwrap(), &tlsh_hash, &ssdeep_hash]).unwrap();
            } else {
                eprint!("Couldn't read file: {:?}, ", _entry.path());
                eprintln!("due to error: {}", try_data.err().unwrap());
            }
        }
    }
}

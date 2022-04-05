use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;
use colored::*;
use rayon::prelude::*;
use regex::Regex;
use separator::Separatable;
use walkdir::WalkDir;

const BOOK_SRC_DIR: &str = "../../src"; // TODO: create proper relative path from env
const WORDS_PER_PAGE: usize = 500;

// Parallel tally of word count by file
// Adapted from: https://da-data.blogspot.com/2020/10/no-c-still-isnt-cutting-it.html
fn scan_chps() -> Result<HashMap<PathBuf, usize>> {
    let mut path_totals = HashMap::new();
    let path_totals_locked = Mutex::new(&mut path_totals);
    let word_regex = Regex::new(r"([a-zA-Z]{1,})")?;

    WalkDir::new(BOOK_SRC_DIR)
        .into_iter()
        .filter_map(Result::ok)
        // Markdown extension name
        .filter(|dir_ent| dir_ent.path().extension().and_then(OsStr::to_str) == Some("md"))
        // Openable
        .map(|dir_entry| (dir_entry.clone(), File::open(dir_entry.path())))
        .filter_map(|(dir_entry, file)| match file {
            Ok(file) => Some((dir_entry, file)),
            _ => None,
        })
        // Actual files
        .filter(|(_, file)| {
            file.metadata()
                .and_then(|meta_data| Ok(meta_data.is_file()))
                .unwrap_or(false)
        })
        .par_bridge()
        .for_each(|(dir_entry, file)| {
            let reader = BufReader::new(file);
            let mut word_count = 0;
            for line in reader.lines().filter_map(Result::ok) {
                let word_list: Vec<String> = word_regex
                    .captures_iter(&line)
                    .map(|w| w[0].to_lowercase())
                    .collect();

                word_count += word_list.len();
            }

            let path = dir_entry
                .path()
                .strip_prefix(BOOK_SRC_DIR)
                .unwrap()
                .to_owned();
            let mut path_totals = path_totals_locked.lock().unwrap();
            path_totals.insert(path, word_count);
        });

    Ok(path_totals)
}

// Print print progress
fn print_results(results: &HashMap<PathBuf, usize>) {
    let mut chp_map: HashMap<&str, Vec<(&str, usize)>> = HashMap::new();

    // Collect: {chp_path : [(sec_1_name, sec_1_size), (sec_2_name, sec_2_size), (sec_3_name, sec_3_size), ... ]}
    for (path, count) in results {
        let chp_key = match path.parent() {
            Some(chp) => match chp.to_str().unwrap() {
                chp if chp == "templates" => continue,
                chp if chp == "" => "Other",
                chp => chp,
            },
            None => "Other",
        };

        if let Some(file) = path.file_name() {
            let chp_files = chp_map.entry(chp_key).or_insert(Vec::new());
            chp_files.push((file.to_str().unwrap(), *count));
        }
    }

    // Sort sections by length
    for (_, files) in chp_map.iter_mut() {
        files.sort_by(|a, b| b.1.cmp(&a.1));
    }

    // Compute chapter numerical order
    let mut sorted_keys: Vec<_> = chp_map.keys().collect();
    sorted_keys.sort_by(|a, b| {
        if a.starts_with("chp") && b.starts_with("chp") {
            let a = a.strip_prefix("chp").unwrap();
            let b = b.strip_prefix("chp").unwrap();

            let a: usize = match a.strip_suffix("_appendix") {
                Some(c) => c.parse().unwrap(),
                None => a.parse().unwrap(),
            };

            let b: usize = match b.strip_suffix("_appendix") {
                Some(c) => c.parse().unwrap(),
                None => b.parse().unwrap(),
            };

            a.cmp(&b)
        } else {
            a.cmp(b)
        }
    });

    // Print chapter totals, sorted by chapter numerical order
    for k in sorted_keys {
        if let Some((chp, files)) = chp_map.get_key_value(k) {
            let total_words: usize = files.iter().map(|e| e.1).sum();

            println!(
                "{}: {} words",
                chp.bright_yellow(),
                total_words.separated_string().bright_green()
            );

            for f in files {
                println!(
                    "- {}: {}",
                    f.0.bright_magenta(),
                    f.1.separated_string().bright_green()
                );
            }

            println!();
        }
    }

    let grand_total: usize = chp_map
        .iter()
        .map(|(_, files)| files.iter().map(|e| e.1).sum::<usize>())
        .sum();

    let page_total = (grand_total / WORDS_PER_PAGE) as usize;

    // Print book total
    println!(
        "{}: {} words ({} pages)\n",
        "BOOK TOTAL".bright_yellow(),
        grand_total.separated_string().bright_green(),
        page_total.separated_string().bright_cyan(),
    );
}

fn main() {
    match scan_chps() {
        Ok(path_map) => print_results(&path_map),
        Err(e) => println!("Error: {:#?}", e),
    };
}

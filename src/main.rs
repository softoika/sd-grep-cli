use clap::{crate_authors, crate_version, App, Arg};
use grep_core::Matcher;
use std::fs::{metadata, File};
use std::io::Read;
use std::path::Path;
use std::thread;

pub struct GrepResult {
    pub file_path: String,
    pub hit_lines: Vec<String>,
}

fn main() {
    let matches = App::new("grep")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Search for PATTERNS in each FILE.")
        .arg(
            Arg::with_name("fixed-strings")
                .short("F")
                .long("fixed-strings")
                .help("PATTERNS are strings."),
        )
        .arg(
            Arg::with_name("PATTERNS")
                .help("use PATTERNS for matching")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("FILES")
                .help("take PATTERNS from FILES")
                .required(true)
                .multiple(true)
                .index(2),
        )
        .get_matches();
    let pattern = matches.value_of("PATTERNS").unwrap();
    let file_pathes = matches
        .values_of("FILES")
        .unwrap()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let is_fixed_string_mode = matches.is_present("fixed-strings");
    let matcher = Matcher::new(pattern.to_string(), is_fixed_string_mode);

    let mut handles = vec![];
    for file_path in file_pathes {
        let matcher = matcher.clone();
        let handle = thread::spawn(move || {
            let path = Path::new(&file_path);
            let display = path.display();
            let mut result = GrepResult {
                file_path: file_path.clone(),
                hit_lines: vec![],
            };
            match metadata(&path) {
                Ok(md) => {
                    if md.is_dir() {
                        return Err(format!("{} is directory", display));
                    }
                }
                Err(e) => {
                    return Err(format!("{}: {}", e.to_string(), display));
                }
            }
            let mut file = match File::open(&path) {
                Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
                Ok(file) => file,
            };
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Err(why) => panic!("couldn't open {}: {}", display, why.to_string()),
                Ok(_) => {
                    for line in s.lines() {
                        if matcher.execute(line) {
                            result.hit_lines.push(line.to_string());
                        }
                    }
                }
            }
            Ok(result)
        });
        handles.push(handle);
    }

    let mut errors = vec![];
    for handle in handles {
        if let Ok(result) = handle.join() {
            match result {
                Ok(result) => {
                    if result.hit_lines.len() > 0 {
                        for line in result.hit_lines {
                            println!("{}:{}", result.file_path, line);
                        }
                    }
                }
                Err(e) => {
                    errors.push(e);
                }
            }
        }
    }
    for e in errors {
        println!("{}", e);
    }
}

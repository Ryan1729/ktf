use grep::{
    matcher::{Match, Matcher},
    regex::RegexMatcherBuilder,
    searcher::{sinks::UTF8, Searcher}
};

use ignore::WalkBuilder;
use std::{
    collections::HashMap,
    cmp::Ordering,
    io::{BufRead, Cursor, Write},
    path::PathBuf
};

mod known_typos;
use known_typos::{FIXES, TYPOS};

struct Typo {
    /// Index into `TYPOS` and `FIXES`.
    index: usize,
    line_number: u64,
    line_match: Match,
    path: PathBuf,
}

impl Ord for Typo {
    fn cmp(&self, other: &Self) -> Ordering {
        // Group typos from the same path together
        self.path.cmp(&other.path)
            // Then group typos from the same line_number together
            .then_with(|| self.line_number.cmp(&other.line_number))
            // Then order typos by where the matches start
            .then_with(|| self.line_match.start().cmp(&other.line_match.start()))
            // We don't expect overlapping matches, but just in case, cmp by end next
            .then_with(|| self.line_match.end().cmp(&other.line_match.end()))
            // As a tie breaker, to maintain partial ordering, cmp by index too.
            .then_with(|| self.index.cmp(&other.index))
    }
}

impl PartialOrd for Typo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Typo {}

impl PartialEq for Typo {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

fn main() {
    let mut searcher = Searcher::new();
    let matcher = RegexMatcherBuilder::new()
        .build_literals(&TYPOS)
        .expect("TYPOS should produce a valid matcher");

    let mut typos = HashMap::with_capacity(16);

    let mut builder = WalkBuilder::new("./");
        builder
        .add_custom_ignore_filename(".ktfignore")
        .skip_stdout(true);
    for walk_result in builder.build() {
        let entry = match walk_result {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("ERROR @ {}:{} : {}", file!(), line!(), err);
                continue
            },
        };

        let path = entry.path();
        if path.is_dir() {
            continue
        }

        let search_result = searcher.search_path(&matcher, &path, UTF8(|line_number, line| {
            let match_result: Result<_, grep::matcher::NoError> = matcher.find_iter(line.as_bytes(), |line_match| {
                let found_typo: &str = &line[line_match];

                // TODO maintain TYPOS in sorted order so we can
                // binary search instead.
                for (index, &typo_str) in TYPOS.iter().enumerate() {
                    if found_typo == typo_str {
                        let vec = typos.entry(path.to_owned())
                            .or_insert_with(|| Vec::with_capacity(16));

                        let typo = Typo {
                            index,
                            line_number,
                            line_match,
                            path: path.to_owned(),
                        };

                        match vec.binary_search(&typo) {
                            Ok(_) => {
                                panic!("Found same typo twice?!");
                            }
                            Err(insert_index) => {
                                vec.insert(insert_index, typo);
                            }
                        }
                        
                        break
                    }
                }

                true
            });

            match match_result {
                Ok(_) => Ok(true),
                Err(_) => {
                    // The err is a `grep::matcher::NoError`, which is documented
                    // to never happen.
                    unreachable!()
                },
            }
        }));

        if let Err(err) = search_result {
            eprintln!("ERROR @ {}:{} : {}", file!(), line!(), err);
        }
    }

    for (path, typo_list) in typos {
        // TODO Do each file in parallel. Maybe with io_uring even?

        let string = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(err) => {
                eprintln!("ERROR @ {}:{} : {}", file!(), line!(), err);
                continue
            }
        };

        let write_result: Result<(), atomicwrites::Error<std::io::Error>> = atomicwrites::AtomicFile::new(
            &path,
            atomicwrites::OverwriteBehavior::AllowOverwrite,
        ).write(|file| {
            let mut cursor = Cursor::new(&string);
            let mut line_number = 0;
    
            let mut line = String::with_capacity(128);
            if let Ok(_) = cursor.read_line(&mut line) {
                assert!(!typo_list.is_empty());
                for typo in typo_list.iter() {
                    loop {
                        match line_number.cmp(&typo.line_number) {
                            Ordering::Equal => {
                                // TODO do the fix by writing the part before the 
                                // typo, the fix, then the rest of the line;
                                file.write(line.as_bytes())?;
                                line_number += 1;
                                line.clear();
                                let Ok(_) = cursor.read_line(&mut line) else {
                                    panic!("We ran out of lines but stil have a typo for {} left?!", path.display());
                                };
                                break
                            }
                            Ordering::Less => {
                                file.write(line.as_bytes())?;
                                line_number += 1;
                                line.clear();
                                let Ok(_) = cursor.read_line(&mut line) else {
                                    panic!("We ran out of lines but stil have a typo for {} left?!", path.display());
                                };
                            }
                            Ordering::Greater => {
                                panic!("We already went past line {} in {} already?!", typo.line_number, path.display());
                            }
                        }
                    }
                }

                // We might have read 0 bytes the last time we read a line. But if
                // so, then writing 0 bytes isn't an issue.
                file.write(line.as_bytes())?;

                line.clear();
                while let Ok(n) = cursor.read_line(&mut line) {
                    file.write(line.as_bytes())?;

                    line.clear();
                    if n == 0 { break }
                }

                Ok(())
            } else {
                panic!("How did we find a typo in an empty file?!");
            }
        });

        if let Err(err) = write_result {
            eprintln!("ERROR @ {}:{} : {}", file!(), line!(), err);
            continue
        };
        println!("\n{}:\n{}", path.display(), string);
    }
}

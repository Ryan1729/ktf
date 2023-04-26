use grep::{
    matcher::{Match, Matcher},
    regex::RegexMatcherBuilder,
    searcher::{sinks::UTF8, Searcher}
};

use ignore::WalkBuilder;
use std::path::PathBuf;

mod known_typos;
use known_typos::{FIXES, TYPOS};

struct Typo {
    /// Index into `TYPOS` and `FIXES`.
    index: usize,
    line_number: u64,
    line_match: Match,
    path: PathBuf,
}

fn main() {
    let mut searcher = Searcher::new();
    let matcher = RegexMatcherBuilder::new()
        .build_literals(&TYPOS)
        .expect("TYPOS should produce a valid matcher");

    let mut typos = Vec::with_capacity(16);

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
                        typos.push(Typo {
                            index,
                            line_number,
                            line_match,
                            path: path.to_owned(),
                        });
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

    for typo in typos {
        println!("{}:{} \"{}\" -> \"{}\" {:?}", typo.path.display(), typo.line_number, TYPOS[typo.index], FIXES[typo.index], typo.line_match);
    }
}

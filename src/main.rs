use grep::{
    matcher::Matcher,
    regex::RegexMatcherBuilder,
    searcher::{sinks::UTF8, Searcher}
};

use ignore::WalkBuilder;

mod known_typos;
use known_typos::{FIXES, TYPOS};

fn main() {
    let mut searcher = Searcher::new();
    let matcher = RegexMatcherBuilder::new()
        .build_literals(&TYPOS)
        .expect("TYPOS should produce a valid matcher");

    let mut builder = WalkBuilder::new("./");
        builder
        .add_custom_ignore_filename(".ktfignore")
        .skip_stdout(true);
    for result in builder.build() {
        let entry = match result {
            Ok(entry) => entry,
            Err(err) => {
                eprintln!("ERROR: {}", err);
                continue
            },
        };

        let path = entry.path();

        searcher.search_path(&matcher, &path, UTF8(|lnum, line| {
            let r#match = matcher.find(line.as_bytes());
            println!("{}:{lnum} {} {:?}", path.display(), line, r#match);

            Ok(true)
        }));
    }
}

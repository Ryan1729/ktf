typo_fix_pairs! {
    ("don;t", "don't"),
    ("won;t", "won't"),
    ("can;t", "can't"),
}

// Wrapped in a module so we can put the invocation above the macro definition,
// in the file.
mod typo_fix_pairs {
    #[macro_export]
    macro_rules! typo_fix_pairs {
        (
            $( ( $typo: literal , $fix: literal $(,)? ) $(,)? )+
        ) => {
            const LENGTH: usize = {
                let mut length = 0;

                $(
                    // Use $typo just so we can use the repetion to do the counting.
                    let _ = $typo;
                    length += 1;
                )*

                length
            };

            const TYPO_FIX_PAIRS: [(&str, &str); LENGTH] = {
                let mut pairs = [
                    $(
                        ($typo, $fix),
                    )*
                ];

                const fn typo_greater_than(a: (&str, &str), b: (&str, &str)) -> bool {
                    let left = a.0.as_bytes();
                    let right = b.0.as_bytes();
                    let short_len = {
                        let mut l = left.len();
                        if right.len() < l {
                            l = right.len();
                        }
                        l
                    };

                    let mut i = 0;
                    while i < short_len {
                        if left[i] != right[i] {
                            return left[i] > right[i]
                        }
                        i += 1;
                    }

                    left.len() > right.len()
                }

                // An insertion sort.
                let mut index = 1;
                while index < pairs.len() {
                    let pair = pairs[index];

                    // Shift things up to make room for `pair`
                    let mut sorted_part_index = index.checked_sub(1);
                    while let Some(spi) = sorted_part_index {
                        if typo_greater_than(pairs[spi], pair) {
                            pairs[spi + 1] = pairs[spi];
                            sorted_part_index = spi.checked_sub(1);
                        } else {
                            break
                        }
                    }

                    let pair_index = match sorted_part_index {
                        Some(spi) => spi + 1,
                        None => 0,
                    };
                    pairs[pair_index] = pair;

                    index += 1;
                }

                pairs
            };
        }
    }

    pub use typo_fix_pairs;
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::OpenOptions;
    use std::io::Write;

    {
        let cwd = std::env::current_dir()?;
        if !cwd.ends_with("tooling/gen") {
            return Err(format!(
                "This program expects to be run in tooling/gen, but it was run in {}",
                cwd.display()
            ).into());
        }
    }

    const PATH: &str = "../../src/known_typos.rs";

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(false)
        .open(PATH)?;

    // TODO? Precalculate the buffer size so we only emit one write syscall.
    write!(file, r#"
///! This file was generated by tooling/gen. Edit the source data and run
///! that generator instead of hand-editing this file.

const LENGTH: usize = {LENGTH};

pub const TYPOS: [&str; LENGTH] = [
"#)?;

    for (typo, _) in TYPO_FIX_PAIRS {
        writeln!(file, "    \"{typo}\",")?;
    }

    write!(file, r#"];

pub const FIXES: [&str; LENGTH] = [
"#)?;

    for (_, fix) in TYPO_FIX_PAIRS {
        writeln!(file, "    \"{fix}\",")?;
    }

    write!(file, r#"];
"#)?;

    println!("Overwrote {PATH} successfully");

    Ok(())
}

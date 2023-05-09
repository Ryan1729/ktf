typo_fix_pairs! {
    "sicne" -> "since",
    "chnage" -> "change",
    "don;t" -> "don't",
    "won;t" -> "won't",
    "can;t" -> "can't",
    "shouldn;t" -> "shouldn't"
    "repetion" -> "repetition",
    "preferrable" -> "preferable",
    "pythoin" -> "python"
    "jsut" -> "just"
    "reasonalbe" -> "reasonable"
    "becasue" ->"because"
    // The space is intended to avoid mangling references to Tehran, for example.
    "teh " -> "the "
    // More commonly seen as "THe", but this should generate that correction
    "tHe " -> "the "
    "verison" -> "version"
    "dispaly" -> "display"
    "witohut" -> "without"
    "dleted" -> "deleted"
    "whoel" -> "whole"
    "inviations" -> "invitations"
    "addiitonal" -> "additional"
    "prinout" -> "printout"
    "enviroment" -> "environment"
    "seeminglt" -> "seemingly"
    "taht" -> "that"
    "expectred" -> "expected"
    "orgnaization" -> "organization"
    "pemissions" -> "permissions"
    "potition" -> "position"
    "tpye" -> "type"
}

// Wrapped in a module so we can put the invocation above the macro definition,
// in the file.
mod typo_fix_pairs {
    #[macro_export]
    macro_rules! typo_fix_pairs {
        (
            $( $typo: literal -> $fix: literal $(,)? )+
        ) => {
            const LENGTH: usize = {
                let mut length = 0;

                $(
                    // Use $typo just so we can use the repetition to do the counting.
                    let _ = $typo;
                    length += 1;
                )*

                length
            };

            const UNNORMALIZED_TYPO_FIX_PAIRS: [(&str, &str); LENGTH] = [
                $(
                    ($typo, $fix),
                )*
            ];
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

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum Transform {
        NoChange,
        UppercaseFirst,
    }

    let unrendered_typo_fixes = {
        let mut unrendered_typo_fixes = Vec::with_capacity(
            UNNORMALIZED_TYPO_FIX_PAIRS.len() * 2
        );

        for (typo, fix) in UNNORMALIZED_TYPO_FIX_PAIRS {
            // Uppercase comes first in ASCII, so put it in the list first.
            unrendered_typo_fixes.push(
                // If this uppercasing turns out to be undesirable in some cases
                // later, we can add a set of flags to each pair, indicating what
                // options should be pushed.
                (typo, fix, Transform::UppercaseFirst)
            );

            unrendered_typo_fixes.push((typo, fix, Transform::NoChange));
        }

        // We rely on this sort being stable so we get the first one only, if we
        // ever do have any duplicates! Having an addition at the end be a no-op
        // seems preferable to potentially different behavior across gen runs with
        // different stdlib versions or whatever.
        unrendered_typo_fixes.sort_by_key(|tuple| tuple.0);
        unrendered_typo_fixes.dedup_by_key(|tuple| (tuple.0, tuple.2));

        unrendered_typo_fixes
    };

    let sorted_typo_fixes = {
        // Since the transforms can and do change the order of the typos when
        // rendered, we must render them before sorting.
        let mut rendered_typo_fixes =
            Vec::with_capacity(unrendered_typo_fixes.len());

        for (typo, fix, transform) in unrendered_typo_fixes.iter() {
            match transform {
                Transform::NoChange => {
                    rendered_typo_fixes.push((
                        format!("{typo}"),
                        format!("{fix}"),
                    ));
                }
                Transform::UppercaseFirst => {
                    rendered_typo_fixes.push((
                        format!("{}", UppercaseFirst(typo)),
                        format!("{}", UppercaseFirst(fix)),
                    ));
                }
            }
        }

        rendered_typo_fixes.sort_by_key(|tuple| tuple.0.clone());
        rendered_typo_fixes.dedup_by_key(|tuple| tuple.0.clone());

        rendered_typo_fixes
    };

    // Assert that each element can be found by binary searching for the typo
    // because the main code relys on being able to do that with the output TYPOS
    // array.
    for element in sorted_typo_fixes.iter() {
        let outcome = sorted_typo_fixes.binary_search_by_key(
            &element.0,
            |tuple| tuple.0.clone()
        );
        assert!(outcome.is_ok(), "Expected Ok(_), got {outcome:?}");
    }

    let length = sorted_typo_fixes.len();

    // TODO? Precalculate the buffer size so we only emit one write syscall.
    write!(file, r#"
///! This file was generated by tooling/gen. Edit the source data and run
///! that generator instead of hand-editing this file.

const LENGTH: usize = {length};

pub const TYPOS: [&str; LENGTH] = [
"#)?;

    for (typo, _) in sorted_typo_fixes.iter() {
        writeln!(file, "    \"{typo}\",")?;
    }

    write!(file, r#"];

pub const FIXES: [&str; LENGTH] = [
"#)?;

    for (_, fix) in sorted_typo_fixes.iter() {
        writeln!(file, "    \"{fix}\",")?;
    }

    write!(file, r#"];
"#)?;

    println!("Overwrote {PATH} successfully");

    Ok(())
}

struct UppercaseFirst(&'static str);

impl core::fmt::Display for UppercaseFirst {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut first = true;
        for ch in self.0.chars() {
            if first {
                for c in ch.to_uppercase() {
                    write!(f, "{c}")?;
                }
                first = false;
                continue
            }
            write!(f, "{ch}")?;
        }
        Ok(())
    }
}

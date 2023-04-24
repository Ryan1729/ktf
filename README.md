# Known Typo Fixer (KTF)

## Overall Goal

Have a spellchecking program that makes sense to run in CI or similar scenarios, like git hooks.

## Completed features

## Desired features

* Reports instances of a small, fixed set of typos in the folder where the program is run
* Expanded set of checked typos
* Have a flag to auto-fix typos that are found, if a fix is known
* Heuristically determine whether a file is a binary file, and leave those alone
* Read de-facto standard .ignore/.gitignore files and respect them, unless a flag is passed

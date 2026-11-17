Read shell-style defaults files. In a `--defaults` file, a line may start
with `export ` (as in `export A=1`), which is ignored, and a value may be
wrapped in one pair of double or single quotes, which are removed. And
`--defaults FILE` may now be given more than once: a later file's
defaults override an earlier one's.

Let `findtext` search for patterns that start with `-`. An argument `--`,
after the optional `-i`, ends the options: everything after it is the
pattern and the files, even when they start with `-`, as in
`findtext -- -v notes.txt` or `findtext -i -- -V notes.txt`. Without `--`,
an argument starting with `-` is still a usage error, and the usage line
now shows `[--]`.

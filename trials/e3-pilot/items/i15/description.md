Skip version-control metadata when sizing a tree. `summarize` and `top_n`
now ignore every directory named `.git`, `.hg`, or `.svn`, with everything
under it: such a directory is not counted in `dirs`, and its files are not
counted in `files` or `bytes` and never appear in `largest` or `top_n`'s
list. The walk takes the list of names to skip as a parameter.

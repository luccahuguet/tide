Add `tail(dir, n)`, which returns the last `n` lines of the log, oldest
first, across `old.2.log`, `old.1.log` and `current.log`, without reading
the whole history: it reads the newest file first and stops once it has
`n` lines. Missing files are skipped as in `read_all`, and other read
errors are returned.

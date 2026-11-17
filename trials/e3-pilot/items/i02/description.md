Write each report atomically. `export` now writes a report's body to a new
temporary `<name>.part` beside it under `out`, then renames that over the
report, so a reader sees the old report or the new one and never part of
one. A stale `<name>.part` is removed first, and the temporary is created
afresh rather than opened, so nothing is written through an existing link.
When the write or the rename fails, the temporary is removed and `export`
returns the error as before. Name checks and the result are unchanged.

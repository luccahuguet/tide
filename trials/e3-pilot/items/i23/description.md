Add `apply_journaled(fs, journal, steps)` for long rename runs. It checks
the steps as `apply` does, then, before each rename, appends the line
`<from>\t<to>` to `renames.journal` in the `journal` directory capability
it is given and syncs it, so that an interrupted run can be finished by
hand. Otherwise it renames exactly as `apply` does. `apply` keeps its
behaviour; the check the two share moves into a private `check`.

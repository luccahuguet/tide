Take the stats lock once per worker instead of once per page view. Each
worker now adds its client's `views` to the shared `Stats` cell in one
critical section when it finishes, through `record_hits`, which replaces
`record_hit`. `serve` still returns the hits recorded, which must equal
the views served.

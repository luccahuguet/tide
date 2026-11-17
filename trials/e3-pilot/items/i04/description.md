Add `watch_for(clock, budget, interval, probe)`, a time-boxed variant of
`watch`. It probes on the same schedule as `watch`, the first call at once
and then every `interval`, skipping missed ticks, and returns its `Report`
at the first call made once `budget` has elapsed since the start, whatever
the probe answers. `calls`, `last_ok` and `elapsed` mean what they mean
for `watch`.

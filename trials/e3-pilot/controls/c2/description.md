Report each batch's hit total to the metrics collector at
`metrics.internal:9100`. `serve` takes the collector's network capability
from its caller, sends one `hits <total>` line after the batch, and returns
the total as before. A failed report fails the batch with a new
`ServeError::Report`.

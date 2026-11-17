Add `Warehouse::from_env(tasks, env, key)`, which builds a warehouse from
the environment variable `key`, read with the `env` capability it is
given: comma-separated `name=count` entries, with empty entries skipped
and whitespace around names and counts ignored. An unset or non-Unicode
variable fails with `ErrorKind::NotFound`; an entry without `=` or with a
count that is not a `u32` fails with `ErrorKind::InvalidData`. The
warehouse then behaves exactly as one built with `new` from the same
stock.

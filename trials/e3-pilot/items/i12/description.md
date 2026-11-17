Support readers that must not change the store. Add
`Store::open_read_only(dir)`, which loads the store as `open` does but
gives a store on which any `set` or `remove` that would save a change
fails with `ErrorKind::PermissionDenied`, without changing anything in
`dir`. Input `set` already rejects is still rejected with
`ErrorKind::InvalidInput`, removing a missing key still returns
`Ok(false)`, and `get` works as usual. Also add `Store::keys()`, the keys
in sorted order, so a reader can list every entry.

Two conveniences for the line server and its client. Add `serve_on(tasks,
net, addr, clients)`, which binds `addr` itself with the `net` capability
it is given and then answers `clients` clients exactly as `serve` does,
returning the same count. And make `exchange` stop when the server closes
the connection: it now returns the replies received so far instead of an
empty string for every remaining line.

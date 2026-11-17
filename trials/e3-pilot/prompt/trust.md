## The trust diff

Tide's compiler also gives you the change's trust diff, below. It compares
the declared authority of the program's exported items before and after
the change: for each item added (`+`), removed (`-`), or changed (`~`),
the capability kinds each parameter path gained (`+`) or lost (`-`). It
also lists `unsafe` sites added or removed. It is partial: it shows only
what exported items' signatures can reach, not what the code does with a
capability it already holds, not how far a capability was narrowed, not
private functions, and not what the output sinks print. An empty trust
diff means no exported item's declared authority changed.

```text
{{TRUST_DIFF}}```

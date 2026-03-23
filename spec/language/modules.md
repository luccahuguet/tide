# Ferox Language Specification: Modules and Visibility

This document defines Ferox's v1 module, import, export, and visibility rules.

## Modules, Imports, Exports, and Visibility

Ferox uses a file-and-module model with explicit imports and explicit public exports.

Design rules:

* Each source file defines one module
* Module boundaries follow the file system layout
* Imports are always explicit
* Visibility is private by default
* Public API surface must be marked explicitly

### Modules

A source file corresponds to a module named from its file.

Examples:

* `math.ferox` defines module `math`
* `net/http.ferox` may define module `net.http`
* `main.ferox` may define the entry module for an executable package

Ferox does not require a `module` declaration in the file in the initial design.

Rationale:

* reduces boilerplate
* keeps the AST predictable
* makes the module graph follow the repository layout directly

### Imports

Imports use `use`.

Import a single item:

```ferox
use net.http.Client;
```

Import a module namespace:

```ferox
use net.http;
```

Import with alias:

```ferox
use net.http.Client as HttpClient;
```

Grouped import:

```ferox
use net.http.{Client, Request, Response};
```

Rules:

* Import paths are dot-separated
* Imports end with `;`
* Aliases use `as`
* Grouped imports are allowed for multiple items from the same module
* Wildcard imports are forbidden in the initial design

### Exports

Public symbols are exported by marking their declarations with `pub`.

Examples:

```ferox
pub fn parse(input: String) -> Result<Value, ParseError> {
    parse_impl(input)
}

fn parse_impl(input: String) -> Result<Value, ParseError> {
    todo();
}
```

Rules:

* `pub` exports a declaration from its defining module
* Unmarked declarations remain private to the module
* Export lists are not used in the initial design
* Public API is declaration-based, not file-footer-based

### Visibility

Initial visibility levels:

* private
* public

Private declaration:

```ferox
fn normalize(value: String) -> String {
    value
}
```

Public declaration:

```ferox
pub fn parse_config(text: String) -> Result<Config, ConfigError> {
    build_config(text)
}
```

Rules:

* Visibility is private by default
* `pub` is the only visibility modifier in the initial design
* More granular visibility such as package-scoped or module-subtree-scoped access is deferred

### Imported Name Usage

Imported names are referenced directly by their local imported name.

Examples:

```ferox
use core.io.Writer;
use core.hash.HashMap as Map;

fn write_all(writer: Writer, values: Map<String, Int>) -> Unit {
    writer.flush();
}
```

Module-qualified references are also allowed when importing a namespace:

```ferox
use net.http;

fn new_client() -> http.Client {
    http.Client::new()
}
```

### Initial Non-Goals

The initial module system does not include:

* wildcard imports
* implicit re-exports
* package-private visibility
* cyclic import support as a design goal
* runtime module loading

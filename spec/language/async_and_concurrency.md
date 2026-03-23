# Ferox Language Specification: Async and Concurrency

This document defines Ferox's v1 async model, ownership-aware structured concurrency rules, and explicit cross-task access model.

## Async Model

Ferox uses explicit async syntax in v1.

Core rule:

* `async fn` marks a function whose execution may suspend and later resume
* async does not imply automatic parallel execution

Initial direction:

* async functions use `async fn`
* the result type in the signature remains the logical result type of the computation, not a user-written future wrapper

Examples:

```ferox
async fn fetch(url: String) -> Response {
    ...
}

async fn render_page(url: String) -> Html {
    let response = await fetch(url);
    render(response)
}
```

Here `fetch` may suspend, and `await` marks the suspension point at the call site.

### Await Syntax

Ferox uses `await expr` as the suspension-point syntax.

Rules:

* `await` may be used only inside an async context
* awaiting an async computation yields its logical result value
* calling an async function in a value position requires `await` unless an explicit concurrency construct captures the computation as child work
* `await` marks a possible suspension point and is therefore always explicit in source

### Model Boundaries

The initial async model does not assume that every async computation is running concurrently.

Rules:

* async controls suspension and resumption semantics
* concurrency primitives that create independent tasks are specified separately
* pure synchronous functions may be called freely from async code
* v1 does not express async through a general effect-set system

### Initial Non-Goals

The initial async model does not include:

* explicit future or promise types in ordinary user-facing function signatures
* implicit async contagion without visible `await`
* a hidden global runtime model baked into the language specification

---

## Concurrency

Goals:

* Safe concurrency by default
* Structured concurrency
* No data races in safe code
* Explicit async behavior

## Concurrency Access Modes

Ferox v1 should expose a small number of explicit cross-task access modes.

The intended modes are:

* ownership transfer
* copy-safe duplication
* explicit concurrent shared ownership

This is the core concurrency simplification:

* local mutation uses ordinary ownership and `&mut`
* cross-task work uses moves, copies of copy-safe values, or explicit `SyncShared[...]`
* ordinary borrows do not silently become a concurrent sharing mechanism

### 1. Ownership Transfer

The preferred cross-task mode is moving owned values into child work.

Example intent:

```ferox
async let response = fetch_owned(request);
```

Here `request` is moved into the child task if it is not copy-safe.

### 2. Copy-Safe Duplication

Copy-safe values may be copied into child work without changing the ownership model.

Example intent:

```ferox
let retries = 3;
async let result = fetch_with_retry(url, retries);
```

This does not create shared mutable state. It only duplicates an ordinary copy-safe value.

### 3. Explicit Concurrent Shared Ownership

When multiple child tasks need access to the same underlying value, concurrency should use explicit shared ownership in the type.

Read-only concurrent sharing:

```ferox
let config = SyncShared::new(Config::load());

async let a = render(config.share());
async let b = index(config.share());
```

Concurrent shared mutation:

```ferox
let state = SyncShared::new(Mutex::new(State::new()));

async let a = update_a(state.share());
async let b = update_b(state.share());
```

Core rule:

* `SyncShared[T]` makes concurrent shared ownership explicit
* mutation still requires an explicit synchronization wrapper such as `Mutex[T]` or `RwLock[T]`
* `SyncShared[T]` alone does not imply shared mutation

## Structured Concurrency

Ferox uses structured concurrency in v1.

Core rule:

* every concurrently running child task must belong to a lexical parent scope

This means child tasks are created inside an enclosing async computation and do not outlive that computation unless a later explicit detached model is added.

### Initial Primitive

Ferox v1 uses one built-in structured-concurrency form:

* `async let` for a small fixed set of child computations

Example intent:

```ferox
async fn load_page(url: String) -> Page {
    async let profile = fetch_profile(url);
    async let posts = fetch_posts(url);

    Page {
        profile: await profile,
        posts: await posts,
    }
}
```

### Parent-Child Rules

Structured concurrency is governed by parent-child lifetime rules.

Rules:

* child tasks begin within a lexical parent scope
* the parent scope is responsible for awaiting or otherwise concluding all child tasks before ordinary successful exit
* leaving a scope with unfinished child tasks is not allowed to silently detach them
* child tasks inherit the enclosing async context rather than creating an unrelated global execution context

### Task Capture Rules

Child-task captures should remain simple and explicit in v1.

Rules:

* capturing a non-copy value into `async let` moves that value unless it is wrapped in an explicit concurrent-sharing type
* capturing a copy-safe value into `async let` copies that value
* ordinary shared borrows from the parent scope do not cross into child tasks by default
* ordinary mutable borrows from the parent scope do not cross into child tasks by default
* values shared across child tasks must use explicit concurrent shared ownership types

This keeps task boundaries visually aligned with ownership boundaries.

### Design Boundaries

The initial structured-concurrency model does not include:

* detached tasks as a default primitive
* unscoped background task spawning
* implicit task creation from every async call
* dynamic task-group constructs in v1

This keeps concurrency visible, bounded, and compatible with Ferox's long-term maintainability goals.

## Early Scope Exit And Unfinished Child Tasks

Ferox v1 does not define a rich general cancellation system.

Core rule:

* unfinished child tasks must not silently detach from their lexical parent scope

Initial direction:

* successful scope completion should not leave unfinished `async let` bindings behind
* early scope exit through `return`, `?`, or other control flow must still preserve structured cleanup rather than detaching child work
* the exact broader cancellation protocol is deferred

### Initial Non-Goals

Ferox v1 does not specify:

* a general cancellation API
* rich cancellation observation rules beyond explicit async suspension points
* sibling-cancellation policy for more advanced task structures
* cancellation-completion protocols for dynamic task groups

## Task Spawning and Synchronization

Task creation and task completion are both explicit in v1.

Core rule:

* spawning child work creates an obligation for the parent scope to synchronize with that work before normal scope completion

### `async let`

`async let` creates a child task immediately when the declaration is evaluated.

Rules:

* `async let name = expr;` starts `expr` as a child task of the current async scope
* the bound name represents the eventual result of that child task
* `await name` synchronizes with the child task and yields its result
* using the bound value without `await` is rejected
* normal successful scope completion with an unfinished `async let` binding is rejected
* early scope exit must still preserve structured cleanup rather than silently detaching the child task

Example:

```ferox
async let profile = fetch_profile(user_id);
let data = await profile;
```

### Synchronization Rules

Rules:

* every `async let` child task must be explicitly awaited or otherwise structurally concluded before ordinary successful scope completion
* synchronization points are explicit: `await name` and structured early-scope cleanup
* the language does not implicitly detach unfinished child tasks on successful scope exit
* child-task results become available only through explicit synchronization operations

## Concurrency Non-Goals For V1

Ferox v1 does not attempt to provide:

* detached background tasks in ordinary safe code
* implicit shared-state concurrency
* borrow-based task capture as an ordinary default
* capability lattices or deep qualifier systems
* rich dynamic task groups
* fully general cancellation semantics

The goal is a visibly bounded concurrency model that remains easy to audit and hard to accidentally widen.

## Concurrency Design Consequences

This concurrency model is intentionally safer to change than ad hoc task and shared-state models.

Current consequences:

* task boundaries align with ownership boundaries instead of hiding them
* code review can distinguish moved ownership, copied small values, and explicit shared-state access from the type surface alone
* mutable shared state is always visually second-class relative to transfer and immutable sharing
* concurrency-related compiler diagnostics can point to a small number of explicit mechanisms instead of many overlapping ones

### Initial Non-Goals

The initial task-spawning model does not include:

* fire-and-forget task spawning in ordinary safe code
* implicit join on successful scope exit without visible synchronization in source
* exposing raw scheduler handles as the default user model
* dynamic task-group constructs in v1

This keeps concurrency mechanics explicit and makes spawned work visible to both humans and tools.

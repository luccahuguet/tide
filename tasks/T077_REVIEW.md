# T077 Review Notes

Review date:

* March 9, 2026

Status:

* started
* first cleanup pass completed

## First-Pass Findings

### 1. Stale status wording remained in the vision and roadmap layer

Problem:

* some top-level docs still read as if the v1 narrowing pass were the current active work
* the repo now says the first narrowing pass is complete and the current recommendation is to begin a narrow Rust bootstrap

Fixes applied:

* updated `spec/VISION.md`
* updated `ROADMAP.md`
* updated `README.md`

### 2. One unsafe-guidance line still referenced arenas as if they were part of the ordinary safe-language path

Problem:

* `spec/LANGUAGE.md` still mentioned arenas in a sentence describing what should stay in safe Ferox
* that was stale after arenas were deferred from the v1 core

Fix applied:

* removed the arena reference from that line

### 3. The tooling doc still had an outdated formatting-deferred note

Problem:

* `spec/TOOLING.md` still said formatter stability guarantees remained for later formatting tasks
* that task has already been completed and the relevant policy is already present later in the same document

Fix applied:

* removed the stale deferred-detail subsection

### 4. One priority label in the vision doc was awkward and lower-quality than the current wording standard

Problem:

* `Agentic-use > human readability` was clumsy and less precise than the rest of the doc

Fix applied:

* replaced it with `Tool and agent analyzability > local terseness`

## Current Assessment

The current spec set is materially cleaner after this pass, but `T077` should remain open.

Areas still worth another read:

* whether the top-level README status text is now exactly calibrated
* whether any comparison or feature-review wording is still broader than the current narrowed v1
* whether the implementation docs have any remaining status language from before `T071`

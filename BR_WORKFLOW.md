# Ferox br Workflow

## Purpose

Ferox uses `br` as the canonical tracker for open work.

The repo docs still carry task meaning, history, and archive material, but open-task status now lives in `br`.

## Canonical Sources

Canonical open-task state:

* `.beads/issues.jsonl`
* the `br` dependency graph and issue metadata

Canonical task meaning and historical archive:

* `tasks/T###.md`
* `TASKS.md`

If task status and repo docs disagree, `br` wins for open work and `TASKS.md` should be updated only when the task is archived as complete.

## Current Scope

Ferox now uses `br` for all open tasks.

The initial seeded queue is:

* `T072`
* `T073`
* `T074`
* `T075`
* `T076`
* `T077`

## Repository-State Rule

The `.beads/` directory is partly committed project state and partly local runtime state.

Commit:

* `.beads/issues.jsonl`
* `.beads/config.yaml`
* `.beads/metadata.json`
* `.beads/.gitignore`

Do not commit:

* `.beads/*.db`
* `.beads/*.db-shm`
* `.beads/*.db-wal`
* lock files
* temp files

## Mapping Rules

When representing repo tasks in `br`:

* keep the `T###` identifier in the issue title
* set the external reference to the matching task doc when possible
* keep task bodies and design context in `tasks/T###.md`
* use `br` status, priority, dependencies, and ready-state for day-to-day execution
* encode real execution dependencies, not every conceptual relationship
* update the task doc when the task meaning changes
* update `TASKS.md` only when archiving completed work

## Daily Use

Useful commands:

* `br ready`
* `br list --pretty`
* `br show <id>`
* `br dep tree <id>`
* `br update <id> --status in_progress`
* `br close <id>`

## Current Working Rule

Use `br` to decide:

* what is open
* what is ready
* what is blocked
* what is in progress
* what is complete

Use the repo docs to decide:

* what the task means
* why the task exists
* what should be preserved as project history

## Completion And Archival

When a numbered task is completed:

1. close it in `br`
2. preserve or update its task body under `tasks/T###.md` if needed
3. add it to `TASKS.md` as a completed archived task

`TASKS.md` should not carry open tasks anymore.

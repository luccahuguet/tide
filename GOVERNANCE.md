# Ferox Governance

## Development Rule - Agent-Produced Code

Ferox development follows an **agent-produced code rule**.

Primary rule:

> No production Ferox code should originate from direct human coding.

Human responsibilities:

* defining architecture
* writing specifications
* reviewing generated code
* validating correctness
* steering project direction

Agent responsibilities:

* implementing compiler components
* writing tooling
* generating tests
* refactoring code
* applying mechanical fixes

Human-written production code is **forbidden by default** and must be explicitly marked if it occurs as an exception.

Philosophy:

> Humans define the system.
> Agents produce the code.

This rule is project governance, not part of the Ferox language specification.

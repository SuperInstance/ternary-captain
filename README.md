# ternary-captain

**Captain/leadership pattern for fleet coordination**

[![ternary](https://img.shields.io/badge/ecosystem-ternary-blue)](https://github.com/orgs/SuperInstance/repositories?q=ternary)
[![tests](https://img.shields.io/badge/tests-22-green)]()

## Overview

Captain/leadership pattern for fleet coordination.

Provides a `Captain` struct that leads a group of agents, a `DecisionEngine`
for weighing ternary options, a `Delegator` for assigning tasks, a
`SituationRoom` for aggregating sensor data, a `FleetReport` for status
aggregation, and a `SuccessionPlan` for captain handoff.

## Architecture

- **`AgentInfo`** — core data structure
- **`DecisionEngine`** — core data structure
- **`Delegator`** — core data structure
- **`SituationRoom`** — core data structure
- **`FleetReport`** — core data structure
- **`SuccessionPlan`** — core data structure
- **`Captain`** — core data structure
- **`Ternary`** — state enumeration
- **`AgentStatus`** — state enumeration

### Key Functions

- `to_i8()`
- `available()`
- `new()`
- `decide()`
- `decide_weighted()`
- `consensus_strength()`
- `new()`
- `assign()`
- `get_assignment()`
- `complete()`
- ... and 28 more

## Why Ternary?

The balanced ternary system {-1, 0, +1} (also known as Z₃) is the mathematically optimal discrete encoding:
- **More expressive than binary**: three states capture positive, neutral, and negative
- **Natural for decisions**: accept/reject/abstain, buy/hold/sell, agree/disagree/neutral
- **Self-balancing**: the 0 state acts as a universal screen, preventing pathological lock-in
- **Z₃ cyclic dynamics**: rock-paper-scissors is the only natural coordination mechanism

## Stats

| Metric | Value |
|--------|-------|
| Lines of Rust | 634 |
| Test count | 22 |
| Public types | 9 |
| Public functions | 38 |

## Ecosystem

This crate is part of the **[SuperInstance Ternary Fleet](https://github.com/orgs/SuperInstance/repositories?q=ternary)**:

- **[ternary-core](https://github.com/SuperInstance/ternary-core)** — shared traits and Z₃ arithmetic
- **[ternary-grid](https://github.com/SuperInstance/ternary-grid)** — spatial grid with {-1, 0, +1} cells
- **[ternary-graph](https://github.com/SuperInstance/ternary-graph)** — ternary-weighted graph algorithms
- **[ternary-automata](https://github.com/SuperInstance/ternary-automata)** — three-state cellular automata
- **[ternary-compiler](https://github.com/SuperInstance/ternary-compiler)** — expression compiler and optimizer

200+ crates. 4,300+ tests. One pattern.

## Research Context

The ternary approach connects to several active research areas:
- **Ternary Neural Networks** (TNNs): weights constrained to {-1, 0, +1} for efficient inference
- **Huawei's ternary chip**: 7nm ternary silicon with 60% less power consumption
- **Active inference**: free energy minimization naturally maps to ternary action selection
- **Cyclic dominance**: RPS dynamics maintain biodiversity in spatial ecology
- **Z₃ group theory**: the only algebraic group on three elements is cyclic addition mod 3

## Usage

```toml
[dependencies]
ternary-captain = "0.1.0"
```

```rust
use ternary_captain;
```

## License

MIT

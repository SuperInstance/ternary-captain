# ternary-captain: Captain/leadership pattern for fleet coordination

## Why This Exists

A fleet of agents needs leadership — not a single point of failure, but a pattern where one agent coordinates others, delegates work, and makes decisions. This crate implements the PLATO captain concept: a lead agent that weighs ternary options, assigns tasks by specialization, aggregates sensor data, and maintains a succession plan for when leadership must transfer.

## Core Concepts

**Captain**: The lead agent. Maintains a roster, makes decisions via the DecisionEngine, delegates tasks, and tracks succession.

**Ternary decisions**: Choices are always three-valued: Negative (reject/retreat), Zero (abstain/gather info), Positive (accept/advance). The DecisionEngine aggregates votes into a fleet decision.

**Quorum**: Minimum number of votes required before a decision is valid. Prevents a single agent from deciding for the fleet.

**Delegation**: Tasks are assigned to agents by matching task type to agent specialization, breaking ties by fitness score.

**Succession plan**: Ordered list of successors. If the captain becomes unavailable, the next in line takes over. Prevents orphaned fleets.

## Quick Start

```toml
[dependencies]
ternary-captain = "0.1"
```

```rust
use ternary_captain::{Captain, AgentInfo, AgentStatus, Ternary, DecisionEngine};

let mut captain = Captain::new("cap-1", 2); // quorum of 2

captain.enlist(AgentInfo {
    id: "scout-1".into(),
    status: AgentStatus::Ready,
    specialization: "recon".into(),
    fitness: 0.9,
});
captain.enlist(AgentInfo {
    id: "medic-1".into(),
    status: AgentStatus::Ready,
    specialization: "medical".into(),
    fitness: 0.7,
});

// Delegate a recon task
let assigned = captain.delegate("patrol-north", "recon");
assert_eq!(assigned, Some("scout-1".to_string()));

// Make a fleet decision
let votes = vec![Ternary::Positive, Ternary::Positive, Ternary::Zero];
let decision = captain.decide_from_votes(&votes);
assert_eq!(decision, Some(Ternary::Positive));
```

## API Overview

| Type | What it is |
|------|-----------|
| `Captain` | Lead agent with roster, decisions, delegation, and succession |
| `DecisionEngine` | Aggregates ternary votes with quorum requirement |
| `Delegator` | Assigns tasks to best-fit agents by specialization and fitness |
| `SituationRoom` | Aggregates ternary sensor reports into fleet-wide picture |
| `FleetReport` | Status rollup: health ratio, operational status, offline agents |
| `SuccessionPlan` | Ordered succession line with promote/remove operations |
| `AgentInfo` | Agent record: id, status, specialization, fitness |
| `AgentStatus` | Ready, Busy, Offline, or Compromised |

## How It Works

The DecisionEngine uses simple majority voting. On ties, the priority order is Negative > Zero > Positive (conservative bias). Weighted voting allows higher-fitness agents to have more influence.

Delegation scans the roster for agents that match the task's specialization and are in Ready status, then picks the highest-fitness match. This is O(n) per delegation — fine for typical fleet sizes but not optimized for thousands of agents.

The SituationRoom collects ternary reports and aggregates them the same way as decisions: majority vote. This gives a fleet-level ternary picture from individual agent perspectives.

Succession is a simple ordered list. When the captain transfers, `promote_next()` removes the heir from the line and the next agent becomes heir. This is deliberate — it models military succession, not democratic election.

## Known Limitations

- **Tie-breaking is arbitrary**: When votes are evenly split, the result depends on enum ordering, not strategic reasoning. Real fleets may need configurable tie-breaking.
- **No vote verification**: The system trusts all votes equally. Malicious agents can manipulate decisions by submitting strategically timed votes.
- **Single-level hierarchy**: No support for captains-of-captains. Deep hierarchies require composing multiple Captain instances manually.
- **No time-based decisions**: Votes aren't timestamped. Late votes count the same as early ones.

## Use Cases

- **Room leadership**: One agent coordinates sensors and actuators in a physical room.
- **Fleet task distribution**: Assign exploration, maintenance, or repair tasks to specialists.
- **Consensus decisions**: Multiple agents vote on whether to enter a room, trigger an alert, or stand down.
- **Graceful leadership transfer**: When a room's captain agent is replaced, the succession plan ensures continuity.

## Ecosystem Context

Part of the SuperInstance ternary ecosystem. The captain pattern maps directly to the PLATO concept and capitaine-1's flagship role:

- `ternary-agent` provides the Agent types that populate the roster
- `ternary-tidelight` synchronizes when captains make decisions (tick-aligned)
- `ternary-flux` can flow decisions through the system
- `ternary-muse` can generate creative ternary patterns for decision exploration

No external dependencies — pure Rust.

## License

MIT

## See Also
- **ternary-consensus** — related fleet coordination
- **ternary-quorum** — related fleet coordination
- **ternary-room** — related fleet coordination
- **ternary-helm** — related fleet coordination
- **ternary-conduct** — related fleet coordination
- **ternary-steward** — related fleet coordination
- **ternary-platoon** — related fleet coordination


# Ternary Captain

**Ternary Captain** implements the captain/leadership pattern for fleet coordination — providing a `Captain` that leads agent groups, a `DecisionEngine` for ternary option weighing, a `Delegator` for task assignment, a `SituationRoom` for sensor aggregation, and a `SuccessionPlan` for captain handoff.

## Why It Matters

Every coordinated fleet needs a leader — but the leader must make decisions in ternary space (proceed +1, abstain 0, reject -1), delegate tasks to the right agents based on specialization, aggregate situational awareness from distributed sensors, and hand off leadership gracefully during failures. Ternary Captain provides all five primitives. It's the fleet-level analogue of `openmind-conductor`'s ensemble coordination, but focused on hierarchical command rather than peer-to-peer orchestration.

## How It Works

### Captain Decision Engine

```
DecisionEngine weighs options:
    for each option:
        compute weighted_sum = Σ (agent_fitness · agent_vote)
        decision = ternary_sign(weighted_sum)
            +1 if sum > threshold
             0 if |sum| ≤ threshold
            -1 if sum < threshold
```

Decision: **O(N)** for N agents voting. The threshold prevents flip-flopping on near-ties.

### Delegation

```rust
delegate(task, agents) → assignment:
    candidates = agents.filter(|a| a.status == Ready && a.specialization matches)
    best = candidates.max_by(fitness)
    assign task to best
```

Delegation: **O(N)** scan of agent list. Assignment tracking enables load balancing.

### Situation Room

Aggregates sensor data from all agents:

```
SituationRoom {
    reports: HashMap<agent_id, SensorReport>,
    aggregate: AggregateSnapshot,
}
```

Update: **O(1)** per agent report. Aggregate computation: **O(N)** per snapshot.

### Succession Planning

When a captain fails, the `SuccessionPlan` determines the next captain:

```
SuccessionPlan {
    order: Vec<agent_id>,  // ranked by fitness + leadership score
    current_index: usize,
}

next_captain() → order[current_index]
```

Succession: **O(1)** (pre-computed order). Re-ranking on fitness changes: **O(N log N)**.

### Fleet Report

```rust
FleetReport {
    total_agents: usize,
    ready: usize,
    busy: usize,
    offline: usize,
    compromised: usize,
    avg_fitness: f64,
}
```

Computation: **O(N)** single pass.

## Quick Start

```rust
use ternary_captain::{Captain, AgentInfo, AgentStatus, Ternary};

let mut captain = Captain::new("fleet-alpha");
captain.add_agent(AgentInfo {
    id: "node-1".into(),
    status: AgentStatus::Ready,
    specialization: "sensor".into(),
    fitness: 0.9,
});

let decision = captain.decide("increase_sampling_rate");
println!("Decision: {:?}", decision); // Positive, Zero, or Negative
```

## API

| Type | Description |
|------|-------------|
| `Captain` | Fleet leader with decision engine and delegation |
| `AgentInfo` | id, status, specialization, fitness |
| `AgentStatus` | Ready, Busy, Offline, Compromised |
| `DecisionEngine` | Ternary decision from weighted votes |
| `Delegator` | Task assignment by specialization and fitness |
| `SituationRoom` | Sensor data aggregation |
| `FleetReport` | Aggregate fleet status |
| `SuccessionPlan` | Captain handoff ordering |
| `Ternary` | Negative (-1), Zero (0), Positive (+1) |

## Architecture Notes

Ternary Captain provides hierarchical leadership for fleet coordination in SuperInstance. In γ + η = C, the captain's Positive (+1) decisions drive γ (growth — fleet-wide expansion and task execution), Negative (-1) decisions implement η (avoidance — rejecting dangerous operations), and Zero (0) maintains equilibrium (wait for more information). Integrates with `ternary-beacon` for captain discovery and `ternary-command` for order dispatch.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for fleet leadership architecture.

## References

1. Lamport, L. (1998). "The Part-Time Parliament." *ACM Transactions on Computer Systems*, 16(2). (Paxos consensus)
2. Oki, B. M. & Liskov, B. (1988). "Viewstamped Replication." *PODC*.
3. Marinescu, D. C. (2017). *Cloud Computing: Theory and Practice*, 2nd ed. Morgan Kaufmann.

## License

MIT

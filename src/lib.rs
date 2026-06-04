#![forbid(unsafe_code)]

//! Captain/leadership pattern for fleet coordination.
//!
//! Provides a `Captain` struct that leads a group of agents, a `DecisionEngine`
//! for weighing ternary options, a `Delegator` for assigning tasks, a
//! `SituationRoom` for aggregating sensor data, a `FleetReport` for status
//! aggregation, and a `SuccessionPlan` for captain handoff.

use std::collections::HashMap;

// ── Ternary Value ──────────────────────────────────────────────────────────

/// A balanced ternary digit: Negative (-1), Zero (0), or Positive (+1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ternary {
    Negative,
    Zero,
    Positive,
}

impl Ternary {
    pub fn to_i8(self) -> i8 {
        match self {
            Ternary::Negative => -1,
            Ternary::Zero => 0,
            Ternary::Positive => 1,
        }
    }
}

// ── Agent Status ───────────────────────────────────────────────────────────

/// Status of an agent in the fleet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentStatus {
    Ready,
    Busy,
    Offline,
    Compromised,
}

impl AgentStatus {
    /// Is this agent available for task assignment?
    pub fn available(self) -> bool {
        self == AgentStatus::Ready
    }
}

// ── Agent Info ─────────────────────────────────────────────────────────────

/// Information about an agent in the fleet.
#[derive(Debug, Clone)]
pub struct AgentInfo {
    pub id: String,
    pub status: AgentStatus,
    pub specialization: String,
    pub fitness: f64,
}

// ── Decision Engine ────────────────────────────────────────────────────────

/// Weighs ternary options and produces decisions.
///
/// Each option is a ternary value. The engine collects votes from agents
/// and produces a final ternary decision.
#[derive(Debug, Clone)]
pub struct DecisionEngine {
    /// Minimum number of votes required for a decision.
    pub quorum: usize,
}

impl DecisionEngine {
    pub fn new(quorum: usize) -> Self {
        Self { quorum }
    }

    /// Decide from a list of ternary votes using majority rule.
    /// Returns None if quorum isn't met.
    pub fn decide(&self, votes: &[Ternary]) -> Option<Ternary> {
        if votes.len() < self.quorum {
            return None;
        }
        let mut counts = [0usize; 3]; // neg, zero, pos
        for &v in votes {
            match v {
                Ternary::Negative => counts[0] += 1,
                Ternary::Zero => counts[1] += 1,
                Ternary::Positive => counts[2] += 1,
            }
        }
        if counts[0] >= counts[1] && counts[0] >= counts[2] {
            Some(Ternary::Negative)
        } else if counts[1] >= counts[0] && counts[1] >= counts[2] {
            Some(Ternary::Zero)
        } else {
            Some(Ternary::Positive)
        }
    }

    /// Decide using weighted votes. Each vote has a weight.
    pub fn decide_weighted(&self, votes: &[(Ternary, f64)]) -> Option<Ternary> {
        if votes.len() < self.quorum {
            return None;
        }
        let mut scores = [0.0f64; 3];
        for &(v, w) in votes {
            match v {
                Ternary::Negative => scores[0] += w,
                Ternary::Zero => scores[1] += w,
                Ternary::Positive => scores[2] += w,
            }
        }
        if scores[0] >= scores[1] && scores[0] >= scores[2] {
            Some(Ternary::Negative)
        } else if scores[1] >= scores[0] && scores[1] >= scores[2] {
            Some(Ternary::Zero)
        } else {
            Some(Ternary::Positive)
        }
    }

    /// Compute consensus strength: ratio of majority votes to total.
    pub fn consensus_strength(&self, votes: &[Ternary]) -> f64 {
        if votes.is_empty() {
            return 0.0;
        }
        let decision = self.decide(votes);
        match decision {
            None => 0.0,
            Some(d) => {
                let majority = votes.iter().filter(|&&v| v == d).count();
                majority as f64 / votes.len() as f64
            }
        }
    }
}

// ── Delegator ──────────────────────────────────────────────────────────────

/// Assigns tasks to agents based on specialization and fitness.
#[derive(Debug, Clone)]
pub struct Delegator {
    /// Pending assignments: task → assigned agent id.
    assignments: HashMap<String, String>,
}

impl Delegator {
    pub fn new() -> Self {
        Self {
            assignments: HashMap::new(),
        }
    }

    /// Assign a task to the best-fit agent from the pool.
    /// Returns the assigned agent's id, or None if no suitable agent found.
    pub fn assign(&mut self, task_id: &str, task_type: &str, agents: &[AgentInfo]) -> Option<String> {
        let best = agents
            .iter()
            .filter(|a| a.status.available())
            .filter(|a| a.specialization == task_type)
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap_or(std::cmp::Ordering::Equal));

        match best {
            Some(agent) => {
                let id = agent.id.clone();
                self.assignments.insert(task_id.to_string(), id.clone());
                Some(id)
            }
            None => None,
        }
    }

    /// Get the agent assigned to a task.
    pub fn get_assignment(&self, task_id: &str) -> Option<&str> {
        self.assignments.get(task_id).map(|s| s.as_str())
    }

    /// Remove a completed assignment.
    pub fn complete(&mut self, task_id: &str) -> bool {
        self.assignments.remove(task_id).is_some()
    }

    /// Number of active assignments.
    pub fn active_count(&self) -> usize {
        self.assignments.len()
    }
}

// ── Situation Room ─────────────────────────────────────────────────────────

/// Aggregates sensor data from agents for decision making.
///
/// Each agent reports a ternary value representing their local situation.
/// The situation room aggregates these into a fleet-wide picture.
#[derive(Debug, Clone)]
pub struct SituationRoom {
    reports: HashMap<String, Ternary>,
}

impl SituationRoom {
    pub fn new() -> Self {
        Self {
            reports: HashMap::new(),
        }
    }

    /// Submit a report from an agent.
    pub fn report(&mut self, agent_id: &str, value: Ternary) {
        self.reports.insert(agent_id.to_string(), value);
    }

    /// Aggregate reports into a single ternary value (majority).
    pub fn aggregate(&self) -> Ternary {
        let votes: Vec<Ternary> = self.reports.values().copied().collect();
        if votes.is_empty() {
            return Ternary::Zero;
        }
        let neg = votes.iter().filter(|&&v| v == Ternary::Negative).count();
        let zero = votes.iter().filter(|&&v| v == Ternary::Zero).count();
        let pos = votes.iter().filter(|&&v| v == Ternary::Positive).count();
        if neg >= zero && neg >= pos {
            Ternary::Negative
        } else if zero >= neg && zero >= pos {
            Ternary::Zero
        } else {
            Ternary::Positive
        }
    }

    /// Number of reports received.
    pub fn report_count(&self) -> usize {
        self.reports.len()
    }

    /// Distribution of reports: (negative_count, zero_count, positive_count).
    pub fn distribution(&self) -> (usize, usize, usize) {
        let neg = self.reports.values().filter(|&&v| v == Ternary::Negative).count();
        let zero = self.reports.values().filter(|&&v| v == Ternary::Zero).count();
        let pos = self.reports.values().filter(|&&v| v == Ternary::Positive).count();
        (neg, zero, pos)
    }

    /// Clear all reports.
    pub fn clear(&mut self) {
        self.reports.clear();
    }
}

// ── Fleet Report ───────────────────────────────────────────────────────────

/// Status aggregation from subordinate agents.
#[derive(Debug, Clone)]
pub struct FleetReport {
    pub agent_reports: HashMap<String, AgentStatus>,
}

impl FleetReport {
    pub fn new() -> Self {
        Self {
            agent_reports: HashMap::new(),
        }
    }

    /// Add an agent's status to the report.
    pub fn add(&mut self, agent_id: &str, status: AgentStatus) {
        self.agent_reports.insert(agent_id.to_string(), status);
    }

    /// Count of agents in each status.
    pub fn status_counts(&self) -> HashMap<AgentStatus, usize> {
        let mut counts = HashMap::new();
        for &status in self.agent_reports.values() {
            *counts.entry(status).or_insert(0) += 1;
        }
        counts
    }

    /// Fleet health: ratio of Ready agents to total.
    pub fn health(&self) -> f64 {
        if self.agent_reports.is_empty() {
            return 0.0;
        }
        let ready = self.agent_reports.values().filter(|&&s| s == AgentStatus::Ready).count();
        ready as f64 / self.agent_reports.len() as f64
    }

    /// Is the fleet operational? (at least one Ready agent and no Compromised).
    pub fn operational(&self) -> bool {
        let has_ready = self.agent_reports.values().any(|&s| s == AgentStatus::Ready);
        let no_compromised = !self.agent_reports.values().any(|&s| s == AgentStatus::Compromised);
        has_ready && no_compromised
    }

    /// Agents that are currently offline.
    pub fn offline_agents(&self) -> Vec<&str> {
        self.agent_reports
            .iter()
            .filter(|(_, &s)| s == AgentStatus::Offline)
            .map(|(id, _)| id.as_str())
            .collect()
    }
}

// ── Succession Plan ────────────────────────────────────────────────────────

/// Handles captain handoff when a room changes or captain becomes unavailable.
#[derive(Debug, Clone)]
pub struct SuccessionPlan {
    /// Ordered list of successors (first = highest priority).
    successors: Vec<String>,
}

impl SuccessionPlan {
    pub fn new() -> Self {
        Self {
            successors: Vec::new(),
        }
    }

    /// Add a successor to the plan.
    pub fn add_successor(&mut self, agent_id: &str) {
        if !self.successors.contains(&agent_id.to_string()) {
            self.successors.push(agent_id.to_string());
        }
    }

    /// Get the current heir (next in line).
    pub fn heir(&self) -> Option<&str> {
        self.successors.first().map(|s| s.as_str())
    }

    /// Remove the current heir (e.g., they became captain) and promote the next.
    pub fn promote_next(&mut self) -> Option<String> {
        if self.successors.is_empty() {
            None
        } else {
            Some(self.successors.remove(0))
        }
    }

    /// Remove an agent from the succession line.
    pub fn remove(&mut self, agent_id: &str) -> bool {
        let idx = self.successors.iter().position(|s| s == agent_id);
        match idx {
            Some(i) => {
                self.successors.remove(i);
                true
            }
            None => false,
        }
    }

    /// Number of successors in line.
    pub fn depth(&self) -> usize {
        self.successors.len()
    }

    /// Full succession line.
    pub fn line(&self) -> &[String] {
        &self.successors
    }
}

// ── Captain ────────────────────────────────────────────────────────────────

/// Leads a group of agents with ternary decision making.
///
/// The captain maintains a roster, makes decisions, delegates tasks, and
/// maintains a succession plan.
#[derive(Debug, Clone)]
pub struct Captain {
    pub id: String,
    pub roster: Vec<AgentInfo>,
    pub decision_engine: DecisionEngine,
    pub delegator: Delegator,
    pub situation_room: SituationRoom,
    pub succession: SuccessionPlan,
}

impl Captain {
    pub fn new(id: &str, quorum: usize) -> Self {
        Self {
            id: id.to_string(),
            roster: Vec::new(),
            decision_engine: DecisionEngine::new(quorum),
            delegator: Delegator::new(),
            situation_room: SituationRoom::new(),
            succession: SuccessionPlan::new(),
        }
    }

    /// Add an agent to the roster.
    pub fn enlist(&mut self, agent: AgentInfo) {
        self.succession.add_successor(&agent.id);
        self.roster.push(agent);
    }

    /// Remove an agent from the roster.
    pub fn discharge(&mut self, agent_id: &str) -> bool {
        let idx = self.roster.iter().position(|a| a.id == agent_id);
        if let Some(i) = idx {
            self.roster.remove(i);
            self.succession.remove(agent_id);
            true
        } else {
            false
        }
    }

    /// Collect votes from all available agents and make a decision.
    pub fn command(&self) -> Option<Ternary> {
        let votes: Vec<Ternary> = self
            .roster
            .iter()
            .filter(|a| a.status.available())
            .map(|_| Ternary::Zero) // placeholder: in real use, agents vote
            .collect();
        self.decision_engine.decide(&votes)
    }

    /// Make a decision from explicit votes.
    pub fn decide_from_votes(&self, votes: &[Ternary]) -> Option<Ternary> {
        self.decision_engine.decide(votes)
    }

    /// Delegate a task to the best-fit agent.
    pub fn delegate(&mut self, task_id: &str, task_type: &str) -> Option<String> {
        self.delegator.assign(task_id, task_type, &self.roster)
    }

    /// Update the situation room with a report.
    pub fn receive_report(&mut self, agent_id: &str, value: Ternary) {
        self.situation_room.report(agent_id, value);
    }

    /// Fleet health based on current roster.
    pub fn fleet_health(&self) -> f64 {
        if self.roster.is_empty() {
            return 0.0;
        }
        let ready = self.roster.iter().filter(|a| a.status.available()).count();
        ready as f64 / self.roster.len() as f64
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_values() {
        assert_eq!(Ternary::Negative.to_i8(), -1);
        assert_eq!(Ternary::Zero.to_i8(), 0);
        assert_eq!(Ternary::Positive.to_i8(), 1);
    }

    #[test]
    fn test_agent_status_available() {
        assert!(AgentStatus::Ready.available());
        assert!(!AgentStatus::Busy.available());
        assert!(!AgentStatus::Offline.available());
    }

    #[test]
    fn test_decision_engine_basic() {
        let engine = DecisionEngine::new(1);
        let votes = vec![Ternary::Positive, Ternary::Positive, Ternary::Negative];
        assert_eq!(engine.decide(&votes), Some(Ternary::Positive));
    }

    #[test]
    fn test_decision_engine_quorum_not_met() {
        let engine = DecisionEngine::new(5);
        let votes = vec![Ternary::Positive, Ternary::Negative];
        assert_eq!(engine.decide(&votes), None);
    }

    #[test]
    fn test_decision_engine_weighted() {
        let engine = DecisionEngine::new(1);
        let votes = vec![(Ternary::Negative, 10.0), (Ternary::Positive, 1.0)];
        assert_eq!(engine.decide_weighted(&votes), Some(Ternary::Negative));
    }

    #[test]
    fn test_consensus_strength_unanimous() {
        let engine = DecisionEngine::new(1);
        let votes = vec![Ternary::Positive, Ternary::Positive, Ternary::Positive];
        assert!((engine.consensus_strength(&votes) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_consensus_strength_split() {
        let engine = DecisionEngine::new(1);
        let votes = vec![Ternary::Positive, Ternary::Negative, Ternary::Zero];
        assert!((engine.consensus_strength(&votes) - (1.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn test_delegator_assign() {
        let mut delegator = Delegator::new();
        let agents = vec![
            AgentInfo { id: "a1".into(), status: AgentStatus::Ready, specialization: "scout".into(), fitness: 0.8 },
            AgentInfo { id: "a2".into(), status: AgentStatus::Ready, specialization: "scout".into(), fitness: 0.9 },
        ];
        let result = delegator.assign("task1", "scout", &agents);
        assert_eq!(result, Some("a2".to_string())); // higher fitness
    }

    #[test]
    fn test_delegator_no_match() {
        let mut delegator = Delegator::new();
        let agents = vec![
            AgentInfo { id: "a1".into(), status: AgentStatus::Ready, specialization: "medic".into(), fitness: 0.9 },
        ];
        assert_eq!(delegator.assign("task1", "scout", &agents), None);
    }

    #[test]
    fn test_delegator_complete() {
        let mut delegator = Delegator::new();
        let agents = vec![
            AgentInfo { id: "a1".into(), status: AgentStatus::Ready, specialization: "scout".into(), fitness: 0.5 },
        ];
        delegator.assign("task1", "scout", &agents);
        assert!(delegator.complete("task1"));
        assert_eq!(delegator.active_count(), 0);
    }

    #[test]
    fn test_situation_room_aggregate() {
        let mut room = SituationRoom::new();
        room.report("a1", Ternary::Positive);
        room.report("a2", Ternary::Positive);
        room.report("a3", Ternary::Negative);
        assert_eq!(room.aggregate(), Ternary::Positive);
    }

    #[test]
    fn test_situation_room_distribution() {
        let mut room = SituationRoom::new();
        room.report("a1", Ternary::Negative);
        room.report("a2", Ternary::Zero);
        room.report("a3", Ternary::Positive);
        assert_eq!(room.distribution(), (1, 1, 1));
    }

    #[test]
    fn test_situation_room_clear() {
        let mut room = SituationRoom::new();
        room.report("a1", Ternary::Positive);
        room.clear();
        assert_eq!(room.report_count(), 0);
    }

    #[test]
    fn test_fleet_report_health() {
        let mut report = FleetReport::new();
        report.add("a1", AgentStatus::Ready);
        report.add("a2", AgentStatus::Ready);
        report.add("a3", AgentStatus::Offline);
        assert!((report.health() - (2.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn test_fleet_report_operational() {
        let mut report = FleetReport::new();
        report.add("a1", AgentStatus::Ready);
        assert!(report.operational());
    }

    #[test]
    fn test_fleet_report_compromised() {
        let mut report = FleetReport::new();
        report.add("a1", AgentStatus::Ready);
        report.add("a2", AgentStatus::Compromised);
        assert!(!report.operational());
    }

    #[test]
    fn test_fleet_report_offline() {
        let mut report = FleetReport::new();
        report.add("a1", AgentStatus::Offline);
        report.add("a2", AgentStatus::Ready);
        assert_eq!(report.offline_agents(), vec!["a1"]);
    }

    #[test]
    fn test_succession_plan() {
        let mut plan = SuccessionPlan::new();
        plan.add_successor("a1");
        plan.add_successor("a2");
        assert_eq!(plan.heir(), Some("a1"));
        assert_eq!(plan.depth(), 2);
    }

    #[test]
    fn test_succession_promote() {
        let mut plan = SuccessionPlan::new();
        plan.add_successor("a1");
        plan.add_successor("a2");
        let promoted = plan.promote_next();
        assert_eq!(promoted, Some("a1".to_string()));
        assert_eq!(plan.heir(), Some("a2"));
    }

    #[test]
    fn test_captain_enlist_and_discharge() {
        let mut captain = Captain::new("cap1", 1);
        captain.enlist(AgentInfo { id: "a1".into(), status: AgentStatus::Ready, specialization: "scout".into(), fitness: 0.9 });
        assert_eq!(captain.roster.len(), 1);
        assert!(captain.discharge("a1"));
        assert_eq!(captain.roster.len(), 0);
    }

    #[test]
    fn test_captain_fleet_health() {
        let mut captain = Captain::new("cap1", 1);
        captain.enlist(AgentInfo { id: "a1".into(), status: AgentStatus::Ready, specialization: "scout".into(), fitness: 0.9 });
        captain.enlist(AgentInfo { id: "a2".into(), status: AgentStatus::Busy, specialization: "medic".into(), fitness: 0.7 });
        assert!((captain.fleet_health() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_captain_delegate() {
        let mut captain = Captain::new("cap1", 1);
        captain.enlist(AgentInfo { id: "a1".into(), status: AgentStatus::Ready, specialization: "scout".into(), fitness: 0.9 });
        let assigned = captain.delegate("task1", "scout");
        assert_eq!(assigned, Some("a1".to_string()));
    }
}

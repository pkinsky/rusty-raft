use std::collections::HashMap;

use types::{LogEntity, LogIdx, NodeId, Term};

// todo: finish impl, quite a few bits here, also will req param over log entry type
pub struct Node {
    pub volatile:  VolatileState,
    pub persisted: PersistedState,
    pub node_type: NodeType,
}

impl Default for Node {
    fn default() -> Node {
        Node {
            volatile: VolatileState::default(),
            persisted: PersistedState::default(),
            node_type: NodeType::Follower,
        }
    }
}

// Reinitialized after election
pub struct VolatileLeaderState {
    // for each server, index of the next log entry to send to that server
    // (initialized to leader last log index + 1)
    next_index:  HashMap<NodeId, LogIdx>,
    // for each server, index of highest log entry known to be replicated on
    // server (initialized to 0, increases monotonically)
    match_index: HashMap<NodeId, LogIdx>,
}

pub struct VolatileState {
    // index of highest log entry known to be committed
    // (initialized to 0, increases monotonically)
    commit_index: LogIdx,
    // index of highest log entry applied to state
    // machine (initialized to 0, increases monotonically)
    last_applied: LogIdx,
}

impl Default for VolatileState {
    fn default() -> VolatileState {
        VolatileState {
            last_applied:  LogIdx(0), //volatile
            commit_index: LogIdx(0), //volatile
        }
    }
}


// Updated on stable storage before responding to RPCs
pub struct PersistedState {
    // latest term server has seen (initialized to 0 on first boot, increases monotonically)
    pub current_term: Term,
    // candidateId that received vote in current term (or null if none)
    pub voted_for: Option<NodeId>,
    // log entries; each entry contains command for state machine, and term when entry
    // was received by leader (first index is 1)
    // NOTE: LogPosition fields of LogEntities must agree with vector index (with 1-start caveat)
    //       formally, they should be monotonically increasing and have no gaps
    pub log: Vec<LogEntity>,
}

impl Default for PersistedState {
    fn default() -> PersistedState {
        PersistedState {
            current_term: Term(0),
            voted_for: None,
            log: Vec::new(),
        }
    }
}

pub enum NodeType {
    Follower,
    Candidate, // todo: candidate-specific state (vote-tracking)
    Leader(VolatileLeaderState),
}
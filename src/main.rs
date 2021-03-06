extern crate actix;
use actix::prelude::*;

use std::collections::HashMap;

use std::time::{SystemTime};
// note: SystemTime::now() is not montonic
// other note: is time even used except for logging (ans: yeah, for clocks, timeouts, etc, right?)

use std::cmp::Ordering;

use std::{thread, time};

// idea: parameterize everything over some log entry type


//TODO: deriving copy on the below newtypes b/c idk wat i doing - justify or remove and fix
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct NodeId(pub u64);

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct LogIdx(pub u64);

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct Term(pub u64);

#[derive(Clone, Copy, Eq, PartialEq)]
struct LogPosition {
    idx: LogIdx,
    term: Term,
}


// Raft determines which of two logs is more up-to-date
//     by comparing the index and term of the last entries in the
//     logs. If the logs have last entries with different terms, then
//     the log with the later term is more up-to-date. If the logs
//     end with the same term, then whichever log is longer is
//     more up-to-date.
impl PartialOrd for LogPosition {
    fn partial_cmp(&self, other: &LogPosition) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogPosition {
    fn cmp(&self, other: &LogPosition) -> Ordering {
        match self.term.cmp(&other.term) {
            Ordering::Equal => self.idx.cmp(&other.idx),
            x => x, // term takes precedence
        }
    }
}

//note: here's how to handle retries/delays:
// ctx.run_later(self.dur, |act, ctx| {
//     // do a thing
// });


struct RequestVote {
    term: Term, // candidate's term
    cid: NodeId, // candidate requesting vote
    last_log_position: LogPosition // idx & term of candidate’s last log entry
}


struct RequestVoteResp {
    term         : Term, // currentTerm, for candidate to update itself
    vote_granted : bool, // true means candidate received vote
}

impl actix::Message for RequestVote {
    type Result = RequestVoteResp;
}


impl Handler<RequestVote> for Node {
    type Result = MessageResult<RequestVote>;

    // Receiver implementation:
    // 1. Reply false if term < currentTerm (§5.1)
    // 2. If votedFor is null or candidateId, and candidate’s log is at
    //    least as up-to-date as receiver’s log, grant vote

    // Raft determines which of two logs is more up-to-date
    //     by comparing the index and term of the last entries in the
    //     logs. If the logs have last entries with different terms, then
    //     the log with the later term is more up-to-date. If the logs
    //     end with the same term, then whichever log is longer is
    //     more up-to-date.

    fn handle(&mut self, req: RequestVote, state: &mut Context<Self>) -> Self::Result {
        // lmao worst name, true if candidates log is >= (at least as up to date) as this node's last entry
        let candidateLogAtLeastAsUpToDate = self.persisted.log.last().map_or(
            LogPosition{idx: LogIdx(0), term: Term(0)},
            |last| {last.position}
        ) <= req.last_log_position;


        // determine if vote can be granted
        let vote_granted =
                req.term >= self.persisted.current_term &&
                self.persisted.voted_for.map_or(true, |cid| {cid == req.cid}) &&
                candidateLogAtLeastAsUpToDate;

        let resp =
            RequestVoteResp {
                term: req.term.max(self.persisted.current_term),
                vote_granted: vote_granted,
            };

        //if req term > current term CONVERT TO FOLLOWER, this node is too behind to be a candidate/leader
        if (req.term > self.persisted.current_term) {
            self.node_type = NodeType::Follower;
        };
        // update own state based on max of terms seen
        self.persisted.current_term = req.term.max(self.persisted.current_term);

        MessageResult(resp)
    }
}



// providing this as a capability so I can spawn actors w/ simulated clock errors via offsets
struct TimeCap {
    get_now: fn() -> SystemTime
}

// helper, removes need to wrap get_now with () before calling
impl TimeCap {
    fn get_now(&self) -> SystemTime {
        (self.get_now)()
    }
}


struct LogEntity {
    position: LogPosition, // term and idx
    value:    String, // todo replace
}

// todo: finish impl, quite a few bits here, also will req param over log entry type
// todo: split out stable, volatile, leader-only state
struct Node {
    volatile:  VolatileState,
    persisted: PersistedState,
    node_type: NodeType,
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
struct VolatileLeaderState {
    // for each server, index of the next log entry to send to that server
    // (initialized to leader last log index + 1)
    next_index:  HashMap<NodeId, LogIdx>,
    // for each server, index of highest log entry known to be replicated on
    // server (initialized to 0, increases monotonically)
    match_index: HashMap<NodeId, LogIdx>,
}

struct VolatileState {
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
struct PersistedState {
    // latest term server has seen (initialized to 0 on first boot, increases monotonically)
    current_term: Term,
    // candidateId that received vote in current term (or null if none)
    voted_for: Option<NodeId>,
    // log entries; each entry contains command for state machine, and term when entry
    // was received by leader (first index is 1)
    // NOTE: LogPosition fields of LogEntities must agree with vector index (with 1-start caveat)
    //       formally, they should be monotonically increasing and have no gaps
    log: Vec<LogEntity>,
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



// todo: can drop Raft prefix here
enum NodeType {
    Follower,
    Candidate, // todo: candidate-specific state (vote-tracking)
    Leader(VolatileLeaderState),
}



impl Actor for Node {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // todo: have some function 'mkTimeCap' that takes, eg, optional offset and provides time cap
        let cap = TimeCap {
            get_now: || { SystemTime::now()}
        };

        println!("now 1 {:?}", cap.get_now());

        // note: never do this in actor, probably sleeps whole system/thread
        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);

        // test confirms diff values, cool. nice to be sure it's not just caching 'now'
        println!("now 2 {:?}", cap.get_now());

        println!("I am alive!");
        System::current().stop(); // <- stop system
    }
}

fn main() {
    let system = System::new("test");

    let addr = Node::default().start();

    system.run();
}

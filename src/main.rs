extern crate actix;
use actix::prelude::*;

use std::time::{SystemTime};
// note: SystemTime::now() is not montonic
// other note: is time even used except for logging (ans: yeah, for clocks, timeouts, etc, right?)


use std::{thread, time};

// idea: parameterize everything over some log entry type
// note: going to need to have nodes capable of handling req, resp for each of 2 req'd rpc types
//       is there a way to keep req/resp cycle in actrix?


//TODO: deriving copy on the below newtypes b/c idk wat i doing - justify or remove and fix
#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct CandidateId(pub u64);

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct LogIdx(pub u64);

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
struct Term(pub u64);


struct RequestVote {
    term          : Term, // candidate's term
    cid           : CandidateId, // candidate requesting vote
    last_log_idx  : LogIdx, // index of candidate’s last log entry
    last_log_term : Term, // term of candidate’s last log entry
}


struct RequestVoteResp {
    term         : Term, // currentTerm, for candidate to update itself
    vote_granted : bool, // true means candidate received vote
}

impl actix::Message for RequestVote {
    type Result = RequestVoteResp;
}



// Persistent state on all servers:
// (Updated on stable storage before responding to RPCs)
//     log[] log entries; each entry contains command
//     for state machine, and term when entry
//     was received by leader (first index is 1)
//     Volatile state on all servers:
// commitIndex index of highest log entry known to be
//     committed (initialized to 0, increases
//                monotonically)
//     lastApplied 
//     Volatile state on leaders:
// (Reinitialized after election)
//     nextIndex[] for each server, index of the next log entry
//     to send to that server (initialized to leader
//                             last log index + 1)
//     matchIndex[] for each server, index of highest log entry
//     known to be replicated on server
//     (initialized to 0, increases monotonically)



impl Handler<RequestVote> for RaftNode {
    type Result = MessageResult<RequestVote>;

    // Receiver implementation:
    // 1. Reply false if term < currentTerm (§5.1)
    // 2. If votedFor is null or candidateId, and candidate’s log is at
    //    least as up-to-date as receiver’s log, grant vote (§5.2, §5.4)

    // TODO: logging goes here
    fn handle(&mut self, req: RequestVote, state: &mut Context<Self>) -> Self::Result {
        // determine if vote can be granted
        let vote_granted =
                req.term >= self.current_term &&
                self.voted_for.map_or(true, |cid| {cid == req.cid}) &&
                // todo: impl to compiling, but cheat sheet is too terse to determine meaning of up-to-date
                //       wrt log (eg: term? idx? applied or committed?)
                self.last_applied >= req.last_log_idx;

        let resp =
            RequestVoteResp {
                term: req.term.max(self.current_term),
                vote_granted: vote_granted,
            };

        // update own state based on term seen (I think this is right - update, it is)
        //TODO: if req term > current term ALSO NEED TO CONVERT TO FOLLOWER
        self.current_term = req.term.max(self.current_term);

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


// todo: finish impl, quite a few bits here, also will req param over log entry type
// todo: split out stable, volatile, leader-only state
struct RaftNode {
    // latest term server has seen (initialized to 0 on first boot, increases monotonically)
    current_term : Term,
    // candidateId that received vote in current term (or null if none)
    voted_for    : Option<CandidateId>,
    // index of highest log entry applied to state machine (initialized to 0, increases monotonically)
    last_applied : LogIdx,
}

impl Default for RaftNode {
    fn default() -> RaftNode {
        RaftNode {
            current_term: Term(0),
            voted_for: None,
            last_applied:  LogIdx(0),
        }
    }
}



impl Actor for RaftNode {
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

    let addr = RaftNode::default().start();

    system.run();
}
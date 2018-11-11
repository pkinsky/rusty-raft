extern crate actix;

use types::{NodeId, LogPosition, Term};

pub struct RequestVote {
    pub term: Term, // candidate's term
    pub cid: NodeId, // candidate requesting vote
    pub last_log_position: LogPosition // idx & term of candidate’s last log entry
}


pub struct RequestVoteResp {
    pub term         : Term, // currentTerm, for candidate to update itself
    pub vote_granted : bool, // true means candidate received vote
}

impl actix::Message for RequestVote {
    type Result = RequestVoteResp;
}

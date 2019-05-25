mod caps;
mod types;

extern crate actix;
use actix::{Actor, Context, Handler, MessageResult};


use std::{thread, time};

use caps::{TimeCap, system_time_cap};
use types::{Term, LogIdx, LogPosition, NodeId};
use types::state::{mk_node, Node, NodeType, VolatileCandidateState};
use types::messages::{RequestVote, RequestVoteResp, TermTimeout};


// todo: need to send this to self on state transition to follower (or as part of init)
// NOTE: this is basically the conversion-to-candidate message? should be in a state transition fn instead tho!
impl Handler<TermTimeout> for Node {
    type Result = MessageResult<TermTimeout>;
    fn handle(&mut self, req: TermTimeout, ctx: &mut Context<Self>) -> Self::Result {
        // need to somehow check timestamp of last heartbeat..
        // if we're not in timeout state just drop out

        // if we're in timeout state then move to candidate
        // (right? iirc there's some random component here to keep them all from going to candidate at the same time)
        let mut candidate_state = VolatileCandidateState::default(); // could just have mkXYZ function to encapsulate insert mut
        candidate_state.votes_received.insert(self.node_id);
        self.node_type = NodeType::Candidate(candidate_state);

        // On conversion to candidate, start election:
        // • Increment currentTerm
        //     • Vote for self
        //     • Reset election timer (I think this just means scheduling another timeout to self msg)
        //     • Send RequestVote RPCs to all other servers
        self.persisted.current_term = self.persisted.current_term.increment();
        self.persisted.voted_for = Some(self.node_id);


        // send msg to all nodes in known node list (note: node id is just an int or something - not an actor addr)
        // so how do I go from one to the other? or do I just use actor id for node id?




        //is it required to even return a msg result?
        MessageResult(())
    }
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

    fn handle(&mut self, req: RequestVote, ctx: &mut Context<Self>) -> Self::Result {
        // lmao worst name, true if candidates log is >= (at least as up to date) as this node's last entry
        let candidate_log_at_least_as_up_to_date = self.persisted.log.last().map_or(
            LogPosition{idx: LogIdx(0), term: Term(0)},
            |last| {last.position}
        ) <= req.last_log_position;


        // determine if vote can be granted
        let vote_granted =
                req.term >= self.persisted.current_term &&
                self.persisted.voted_for.map_or(true, |cid| {cid == req.cid}) &&
                candidate_log_at_least_as_up_to_date;

        let resp =
            RequestVoteResp {
                term: req.term.max(self.persisted.current_term),
                vote_granted: vote_granted,
            };

        //if req term > current term CONVERT TO FOLLOWER, this node is too behind to be a candidate/leader
        if req.term > self.persisted.current_term {
            self.node_type = NodeType::Follower;
        };
        // update own state based on max of terms seen
        self.persisted.current_term = req.term.max(self.persisted.current_term);

        MessageResult(resp)
    }
}

impl Actor for Node {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let cap = system_time_cap();
        println!("started node with id {:?} at {:?}", self.node_id, cap.get_now());

        // note: never do this in actor, probably sleeps whole system/thread
        let ten_millis = time::Duration::from_millis(10);
        thread::sleep(ten_millis);

        // test confirms diff values, cool. nice to be sure it's not just caching 'now'
        println!("now 2 {:?}", cap.get_now());

        println!("I am alive!");
        actix::System::current().stop(); // <- stop system
    }
}

fn main() {
    let system = actix::System::new("test");

    let id = NodeId(1);
    let addr = mk_node(id).start();

    system.run();
}

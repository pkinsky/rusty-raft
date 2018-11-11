mod caps;
mod types;

extern crate actix;
use actix::{Actor, Context, Handler, MessageResult};


use std::time::{SystemTime};
// note: SystemTime::now() is not montonic
// other note: is time even used except for logging (ans: yeah, for clocks, timeouts, etc, right?)

use std::{thread, time};

use caps::{TimeCap};
use types::{Term, LogIdx, LogPosition};
use types::state::{Node, NodeType};
use types::messages::{RequestVote, RequestVoteResp};

//note: here's how to handle retries/delays:
// ctx.run_later(self.dur, |act, ctx| {
//     // do a thing
// });

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
        actix::System::current().stop(); // <- stop system
    }
}

fn main() {
    let system = actix::System::new("test");

    let addr = Node::default().start();

    system.run();
}

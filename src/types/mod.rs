pub mod state;
pub mod messages;

use std::cmp::Ordering;
use std::hash::Hash;

//TODO: deriving copy on the below newtypes b/c idk wat i doing - justify or remove and fix
#[derive(Clone, Copy, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct NodeId(pub u64);

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct LogIdx(pub u64);

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub struct Term(pub u64);

// note: is this overly tied to immutable/functional style? Could just mutate in place instead of returning new
impl Term {
    pub fn increment(&self) -> Term {
        let Term(u) = self;
        Term(u + 1)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LogPosition {
    pub idx: LogIdx,
    pub term: Term,
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


pub struct LogEntity {
    pub position: LogPosition, // term and idx
    pub value:    String, // todo replace
}

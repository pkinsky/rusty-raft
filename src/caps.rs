

use std::time::{SystemTime};
// note: SystemTime::now() is not montonic
// other note: is time even used except for logging (ans: yeah, for clocks, timeouts, etc, right?)


pub struct TimeCap {
    pub get_now: fn() -> SystemTime
}

// helper, removes need to wrap get_now with () before calling
impl TimeCap {
    pub fn get_now(&self) -> SystemTime {
        (self.get_now)()
    }
}

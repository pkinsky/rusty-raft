// note: this exists only as the result of an experiment in capabilities style in rust & will probably not be used

use std::time::{SystemTime};


pub struct TimeCap {
    pub get_now: fn() -> SystemTime
}

// helper, removes need to wrap get_now with () before calling
impl TimeCap {
    pub fn get_now(&self) -> SystemTime {
        (self.get_now)()
    }
}

// todo: have some function 'mkTimeCap' that takes, eg, optional offset and provides time cap
pub fn system_time_cap() -> TimeCap {
    TimeCap {
        get_now: || { SystemTime::now()}
    }
}

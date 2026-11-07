use anchor_lang::prelude::*;

pub fn now_unix_seconds() -> u64 {
    Clock::get().unwrap().unix_timestamp.try_into().unwrap()
}

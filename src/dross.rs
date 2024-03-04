use crate::faery::Model;

// transfer_dross takes a sender and a receiver and an amount of dross to transfer.
// It returns a Result that is Ok(()) if the transfer was successful and Err(()) if it was not.
#[allow(dead_code)]
pub fn transfer_dross(sender: &mut Model, receiver: &mut Model, amount: u32) -> DrossResult {
    match sender.decrement_dross(amount) {
        Ok(_) => {
            receiver.increment_dross(amount)
        },
        Err(e) => {
            Err(e)
        }
    }
}

pub trait DrossHolder {
    fn increment_dross(&mut self, amount: u32) -> DrossResult;
    fn decrement_dross(&mut self, amount: u32) -> DrossResult;
    fn dross(&self) -> DrossResult;
}

pub type DrossResult = Result<u32, DrossError>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum DrossError {
    // NegativeDross,
    NotEnoughDross,
    InvalidIncrement,
    InvalidDecrement,
}

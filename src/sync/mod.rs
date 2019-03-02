pub mod queue;
pub mod signal;
pub mod broadcast;

#[derive(Clone, Copy)]
/// Queue / Signal capacity
pub enum Capacity {
    /// Unlimited number of messages
    Unbounded,
    /// Limited number of messages
    Bounded(usize),
}

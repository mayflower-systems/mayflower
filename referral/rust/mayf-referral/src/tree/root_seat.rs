use crate::Amount;

/// An authenticated result of the protocol's scarce-seat auction.
/// The host stores this record under its auction allocation key, and supplies
/// it only to the winning owner. Activation consumes it and burns the price.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RootSeat<I> {
    Awarded { node_id: I, price: Amount },
    Activated,
}

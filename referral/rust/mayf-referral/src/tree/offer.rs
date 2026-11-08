use crate::Amount;

/// The price of a seat: a node's price for new children, or the protocol's price for top-level
/// members. Each join charges the current fee, then raises it by the increment, up to
/// `Amount::MAX`. A fee of 10 with an increment of 2 charges 10, 12, then 14.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Offer {
    pub current_fee: Amount,
    pub increment: Amount,
}

impl Offer {
    /// Charge the current fee, then raise it by the increment.
    /// Returns the raised offer, and the fee charged.
    pub(crate) fn charge(self) -> (Self, Amount) {
        let raised = Self {
            current_fee: self.current_fee.saturating_add(self.increment),
            ..self
        };
        (raised, self.current_fee)
    }
}

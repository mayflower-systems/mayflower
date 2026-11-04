use crate::state::Market;
use mayf_fixed_number::{FixedNumber, Pos};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DonateReserveArgs<N: FixedNumber> {
    pub amount_reserve: Pos<N>,
    pub market: Market<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DonateReserveReceipt<N: FixedNumber> {
    pub market_next: Market<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DonateReserveError {
    ReserveBalanceOverflow,
}

/// Add normalized reserve without changing supply, segmentation, or advances.
pub fn donate_reserve<N>(
    args: DonateReserveArgs<N>,
) -> Result<DonateReserveReceipt<N>, DonateReserveError>
where
    N: FixedNumber,
{
    let Market { reserve_held, .. } = args.market;

    let new_reserve_held = reserve_held
        .checked_add(args.amount_reserve)
        .map_err(|_| DonateReserveError::ReserveBalanceOverflow)?;

    let market_next = Market {
        reserve_held: new_reserve_held,
        ..args.market
    };

    Ok(DonateReserveReceipt { market_next })
}

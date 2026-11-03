use crate::state::Market;
use mayf_fixed_number::FixedNumber;

pub struct DonateReserveArgs<N>
where
    N: FixedNumber,
{
    pub amount_reserve: u64,
    pub market: Market<N>,
}

pub struct DonateReserveReceipt<N>
where
    N: FixedNumber,
{
    /// the next state of the market
    pub market_next: Market<N>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DonateReserveError {
    ReserveBalanceOverflow,
}

/// Donate reserve units to the market
pub fn donate_reserve<N>(
    args: DonateReserveArgs<N>,
) -> Result<DonateReserveReceipt<N>, DonateReserveError>
where
    N: FixedNumber,
{
    let Market { reserve_held, .. } = args.market;

    let new_reserve_held = reserve_held
        .checked_add(args.amount_reserve)
        .ok_or(DonateReserveError::ReserveBalanceOverflow)?;

    let market_next = Market {
        reserve_held: new_reserve_held,
        ..args.market
    };

    Ok(DonateReserveReceipt { market_next })
}

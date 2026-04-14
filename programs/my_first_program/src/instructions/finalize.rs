use anchor_lang::prelude::*;
use crate::state::{ProgramConfig, Market, MarketState, DISPUTE_WINDOW};
use crate::errors::EscrowError;
use crate::events::MarketFinalized;

/// Admin finalises the market and locks the fee/prize math.
///
/// From OutcomeProposed: requires 3-minute dispute window to have elapsed.
/// From Disputed: admin can finalise immediately after off-chain review,
///                and may supply a different winning_side than originally proposed.
///
/// After finalize():
///   - prize_pool         = 85 % of total_pool  → winners claim proportionally
///   - creator_owed       =  5 % of total_pool  → creator claims via claim_creator
///   - platform_fee_total = 10 % of total_pool  → attributed lazily by process_split_*
///
/// Integer division is used throughout; the rounding benefit goes to the prize
/// pool (winners) so that platform_fee_total + creator_owed + prize_pool == total_pool.
pub fn handler(ctx: Context<Finalize>, winning_side: bool) -> Result<()> {
    let now    = Clock::get()?.unix_timestamp;
    let market = &mut ctx.accounts.market;

    match &market.state {
        MarketState::OutcomeProposed { proposed_at, .. } => {
            require!(now >= proposed_at + DISPUTE_WINDOW, EscrowError::DisputeWindowNotExpired);
        }
        MarketState::Disputed { .. } => {
            // Admin overrides after off-chain review — no time gate.
        }
        _ => return Err(EscrowError::InvalidMarketState.into()),
    }

    let winning_total = if winning_side { market.yes_total } else { market.no_total };
    require!(winning_total > 0, EscrowError::NoWinningSide);

    let total_pool = market.yes_total
        .checked_add(market.no_total)
        .ok_or(EscrowError::Overflow)?;

    let platform_fee_total = total_pool / 10;       // floor → 10 %
    let creator_owed       = total_pool * 5 / 100;  // floor →  5 %
    let prize_pool         = total_pool
        .checked_sub(platform_fee_total)
        .and_then(|v| v.checked_sub(creator_owed))
        .ok_or(EscrowError::Overflow)?;             // remainder → 85 %

    let market_id = market.market_id;
    market.total_pool         = total_pool;
    market.prize_pool         = prize_pool;
    market.creator_owed       = creator_owed;
    market.platform_fee_total = platform_fee_total;
    market.state              = MarketState::Finalized { side: winning_side };

    emit!(MarketFinalized {
        market_id,
        winning_side,
        prize_pool,
        creator_owed,
        platform_fee_total,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(seeds = [b"config"], bump = config.bump, has_one = admin)]
    pub config: Account<'info, ProgramConfig>,

    pub admin: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,
}

use anchor_lang::prelude::*;
use crate::state::{Market, MarketState, DISPUTE_WINDOW};
use crate::errors::EscrowError;
use crate::events::DisputeRaised;

/// Any signed wallet can raise a dispute within 3 minutes of outcome proposal.
///
/// The state moves from OutcomeProposed → Disputed.  The admin reviews the
/// dispute off-chain and then calls finalize() with the correct result.
pub fn handler(ctx: Context<Dispute>) -> Result<()> {
    let now    = Clock::get()?.unix_timestamp;
    let market = &mut ctx.accounts.market;

    let market_id = market.market_id;

    match &market.state {
        MarketState::OutcomeProposed { side, proposed_at } => {
            require!(now < proposed_at + DISPUTE_WINDOW, EscrowError::DisputeWindowExpired);
            let side        = *side;
            let proposed_at = *proposed_at;
            market.state    = MarketState::Disputed { side, proposed_at };
        }
        _ => return Err(EscrowError::InvalidMarketState.into()),
    }

    emit!(DisputeRaised { market_id, disputer: ctx.accounts.disputer.key(), at: now });
    Ok(())
}

#[derive(Accounts)]
pub struct Dispute<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    /// Any wallet can dispute; no position required.
    pub disputer: Signer<'info>,
}

use anchor_lang::prelude::*;
use crate::state::{ProgramConfig, Market, MarketState, SWITCH_WINDOW};
use crate::errors::EscrowError;
use crate::events::OutcomeProposed;

/// Admin proposes the winning side after the 30-second lock.
///
/// This implicitly locks the pool: yes_total and no_total are frozen from this
/// point onwards.  A 3-minute dispute window then opens.
pub fn handler(ctx: Context<ProposeOutcome>, winning_side: bool) -> Result<()> {
    let now    = Clock::get()?.unix_timestamp;
    let market = &mut ctx.accounts.market;

    require!(matches!(market.state, MarketState::Open), EscrowError::InvalidMarketState);
    require!(now >= market.open_ts + SWITCH_WINDOW, EscrowError::MarketNotLocked);

    let market_id = market.market_id;
    market.state  = MarketState::OutcomeProposed { side: winning_side, proposed_at: now };

    emit!(OutcomeProposed { market_id, side: winning_side, proposed_at: now });
    Ok(())
}

#[derive(Accounts)]
pub struct ProposeOutcome<'info> {
    #[account(seeds = [b"config"], bump = config.bump, has_one = admin)]
    pub config: Account<'info, ProgramConfig>,

    pub admin: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,
}

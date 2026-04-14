use anchor_lang::prelude::*;
use crate::state::{ProgramConfig, Market, MarketState};
use crate::errors::EscrowError;
use crate::events::MarketVoided;

/// Admin voids the market — all user stakes become fully refundable.
///
/// Admin can void from Open, OutcomeProposed, or Disputed states.
/// A Finalized market cannot be voided.
pub fn handler(ctx: Context<VoidMarket>) -> Result<()> {
    let market    = &mut ctx.accounts.market;
    let market_id = market.market_id;

    require!(
        matches!(
            market.state,
            MarketState::Open
                | MarketState::OutcomeProposed { .. }
                | MarketState::Disputed { .. }
        ),
        EscrowError::InvalidMarketState,
    );

    market.state = MarketState::Voided;
    emit!(MarketVoided { market_id });
    Ok(())
}

#[derive(Accounts)]
pub struct VoidMarket<'info> {
    #[account(seeds = [b"config"], bump = config.bump, has_one = admin)]
    pub config: Account<'info, ProgramConfig>,

    pub admin: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,
}

use anchor_lang::prelude::*;
use crate::state::{Market, UserPosition};
use crate::errors::EscrowError;
use crate::events::SideSwitched;

/// Switch the user's entire position from YES → NO or NO → YES.
///
/// - Only callable during seconds 15–30 of the market (switching window).
/// - No USDC moves; the funds remain in the vault.  Only the on-chain ledger
///   counters are updated so the pool sizes stay accurate.
/// - A user may switch multiple times within the 30-second window.
pub fn handler(ctx: Context<SwitchSide>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(ctx.accounts.market.is_switching_open(now), EscrowError::SwitchingWindowClosed);

    let market   = &mut ctx.accounts.market;
    let position = &mut ctx.accounts.user_position;
    let market_id = market.market_id;

    if position.yes_stake > 0 {
        // YES → NO
        let stake = position.yes_stake;
        market.yes_total = market.yes_total.checked_sub(stake).ok_or(EscrowError::Overflow)?;
        market.no_total  = market.no_total .checked_add(stake).ok_or(EscrowError::Overflow)?;
        position.no_stake  = stake; 
        position.yes_stake = 0;
        emit!(SideSwitched { market_id, user: ctx.accounts.user_wallet.key(), from_yes: true,  amount: stake });
    } else if position.no_stake > 0 {
        // NO → YES
        let stake = position.no_stake;
        market.no_total  = market.no_total .checked_sub(stake).ok_or(EscrowError::Overflow)?;
        market.yes_total = market.yes_total.checked_add(stake).ok_or(EscrowError::Overflow)?;
        position.yes_stake = stake;
        position.no_stake  = 0;
        emit!(SideSwitched { market_id, user: ctx.accounts.user_wallet.key(), from_yes: false, amount: stake });
    } else {
        return Err(EscrowError::NoStakeToSwitch.into());
    }

    Ok(())
}

#[derive(Accounts)]
pub struct SwitchSide<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"position", market.key().as_ref(), user_wallet.key().as_ref()],
        bump  = user_position.bump,
    )]
    pub user_position: Account<'info, UserPosition>,

    pub user_wallet: Signer<'info>,
}

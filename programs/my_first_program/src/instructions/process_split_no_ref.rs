use anchor_lang::prelude::*;
use crate::state::{Market, MarketState, User, UserPosition};
use crate::errors::EscrowError;
use crate::events::PlatformSplitProcessed;

/// Attribute the platform fee share for a user who has NO referrer.
///
/// 100 % of this user's slice of the platform fee goes to the platform.
///
/// This is a permissionless, idempotent instruction — the relayer/indexer
/// calls it once per position after finalization.  It does NOT transfer any
/// tokens; it only updates `market.platform_accumulated` so that
/// `claim_platform` can eventually sweep the total.
///
/// Must NOT be called for users who have a referrer (use process_split_referred).
pub fn handler(ctx: Context<ProcessSplitNoRef>) -> Result<()> {
    require!(
        matches!(ctx.accounts.market.state, MarketState::Finalized { .. }),
        EscrowError::NotFinalized,
    );

    let position = &mut ctx.accounts.user_position;
    require!(!position.platform_split_done, EscrowError::SplitAlreadyDone);

    // Guard: this instruction is only for users without a referrer.
    let user_account = &ctx.accounts.user_account;
    require!(user_account.referrer.is_none(), EscrowError::UserHasReferrer);

    let total_pool         = ctx.accounts.market.total_pool;
    let platform_fee_total = ctx.accounts.market.platform_fee_total;
    let market_id          = ctx.accounts.market.market_id;
    let user_key           = ctx.accounts.user_wallet.key();

    // user's contribution = (stake / total_pool) * platform_fee_total
    let stake = position.total_stake();
    let user_slice = if total_pool > 0 {
        (stake as u128)
            .checked_mul(platform_fee_total as u128).ok_or(EscrowError::Overflow)?
            .checked_div(total_pool as u128).ok_or(EscrowError::Overflow)? as u64
    } else {
        0
    };

    position.platform_split_done = true;

    let market = &mut ctx.accounts.market;
    market.platform_accumulated = market.platform_accumulated
        .checked_add(user_slice)
        .ok_or(EscrowError::Overflow)?;

    emit!(PlatformSplitProcessed {
        market_id,
        user: user_key,
        platform_share:  user_slice,
        affiliate_share: 0,
        referrer:        None,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct ProcessSplitNoRef<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"position", market.key().as_ref(), user_wallet.key().as_ref()],
        bump  = user_position.bump,
    )]
    pub user_position: Account<'info, UserPosition>,

    #[account(
        seeds = [b"user", user_wallet.key().as_ref()],
        bump  = user_account.bump,
    )]
    pub user_account: Account<'info, User>,

    /// CHECK: Public key only — used for PDA seed derivation; does not need to sign.
    pub user_wallet: AccountInfo<'info>,
}

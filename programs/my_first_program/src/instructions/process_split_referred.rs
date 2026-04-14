use anchor_lang::prelude::*;
use crate::state::{Market, MarketState, User, UserPosition, AffiliateClaim};
use crate::errors::EscrowError;
use crate::events::PlatformSplitProcessed;

/// Attribute the platform fee share for a user who HAS a referrer.
///
/// - 30 % of this user's platform-fee slice → affiliate_claim (per market/referrer PDA)
/// - 70 % → market.platform_accumulated (claimable by the platform later)
///
/// Permissionless and idempotent — the relayer calls it once per referred position
/// after finalization.  On the first call the AffiliateClaim PDA is created (if it
/// doesn't exist yet) via init_if_needed; subsequent calls for other users with the
/// same referrer just increment amount_owed.
///
/// Must NOT be called for users without a referrer (use process_split_no_ref).
pub fn handler(ctx: Context<ProcessSplitReferred>) -> Result<()> {
    require!(
        matches!(ctx.accounts.market.state, MarketState::Finalized { .. }),
        EscrowError::NotFinalized,
    );

    let position = &mut ctx.accounts.user_position;
    require!(!position.platform_split_done, EscrowError::SplitAlreadyDone);

    // Guard: caller must pass the user's actual stored referrer.
    let user_account = &ctx.accounts.user_account;
    let stored_referrer = user_account.referrer.ok_or(EscrowError::UserHasNoReferrer)?;
    require_keys_eq!(
        ctx.accounts.referrer.key(),
        stored_referrer,
        EscrowError::InvalidReferrer,
    );

    let total_pool         = ctx.accounts.market.total_pool;
    let platform_fee_total = ctx.accounts.market.platform_fee_total;
    let market_id          = ctx.accounts.market.market_id;
    let user_key           = ctx.accounts.user_wallet.key();
    let referrer_key       = ctx.accounts.referrer.key();
    let market_key         = ctx.accounts.market.key();

    let stake = position.total_stake();
    let user_slice = if total_pool > 0 {
        (stake as u128)
            .checked_mul(platform_fee_total as u128).ok_or(EscrowError::Overflow)?
            .checked_div(total_pool as u128).ok_or(EscrowError::Overflow)? as u64
    } else {
        0
    };

    let affiliate_share = user_slice * 30 / 100;
    let platform_share  = user_slice - affiliate_share; // platform gets rounding benefit

    position.platform_split_done = true;

    // ── Init or update AffiliateClaim ─────────────────────────────────────
    let claim = &mut ctx.accounts.affiliate_claim;
    if claim.market == Pubkey::default() {
        claim.market   = market_key;
        claim.referrer = referrer_key;
        claim.bump     = ctx.bumps.affiliate_claim;
    }
    claim.amount_owed = claim.amount_owed
        .checked_add(affiliate_share)
        .ok_or(EscrowError::Overflow)?;

    // ── Update market platform accumulator ───────────────────────────────
    let market = &mut ctx.accounts.market;
    market.platform_accumulated = market.platform_accumulated
        .checked_add(platform_share)
        .ok_or(EscrowError::Overflow)?;

    emit!(PlatformSplitProcessed {
        market_id,
        user:            user_key,
        platform_share,
        affiliate_share,
        referrer:        Some(referrer_key),
    });
    Ok(())
}

#[derive(Accounts)]
pub struct ProcessSplitReferred<'info> {
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

    /// CHECK: Used for PDA seed derivation; identity verified in handler.
    pub user_wallet: AccountInfo<'info>,

    /// CHECK: The referrer's wallet pubkey; validated against user_account.referrer.
    pub referrer: AccountInfo<'info>,

    /// AffiliateClaim PDA — created on first call for this (market, referrer) pair.
    #[account(
        init_if_needed,
        payer  = payer,
        space  = AffiliateClaim::SPACE,
        seeds  = [b"affiliate", market.key().as_ref(), referrer.key().as_ref()],
        bump,
    )]
    pub affiliate_claim: Account<'info, AffiliateClaim>,

    /// Pays for AffiliateClaim account rent on first call (typically the relayer).
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

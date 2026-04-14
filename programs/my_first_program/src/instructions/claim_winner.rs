use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, TransferChecked};
use crate::state::{Market, MarketState, UserPosition};
use crate::errors::EscrowError;
use crate::events::WinnerClaimed;

/// Winner claims their proportional share of the prize pool.
///
/// Payout formula (integer arithmetic, u128 intermediate):
///   payout = (user_stake * prize_pool) / winning_side_total
///
/// This returns the user's original stake plus their share of the losing pool
/// minus the 15 % platform+creator fees.  In the degenerate case where the
/// winning side is the only side, the winner receives only 85 % of their stake.
pub fn handler(ctx: Context<ClaimWinner>) -> Result<()> {
    // Read all market fields before any mutable borrow of position.
    let winning_side = match &ctx.accounts.market.state {
        MarketState::Finalized { side } => *side,
        _ => return Err(EscrowError::NotFinalized.into()),
    };

    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let market_bump     = ctx.accounts.market.bump;
    let prize_pool      = ctx.accounts.market.prize_pool;
    let winning_total   = ctx.accounts.market.winning_total()
        .ok_or(EscrowError::NotFinalized)?;
    require!(winning_total > 0, EscrowError::NoWinningSide);

    // Validate and lock position.
    let position = &mut ctx.accounts.user_position;
    require!(!position.claimed_winnings, EscrowError::AlreadyClaimed);
    let user_stake = position.stake_on_side(winning_side);
    require!(user_stake > 0, EscrowError::NotAWinner);

    // u128 to avoid overflow when prize_pool * stake could exceed u64::MAX.
    let payout_u128 = (user_stake as u128)
        .checked_mul(prize_pool as u128)
        .ok_or(EscrowError::Overflow)?
        .checked_div(winning_total as u128)
        .ok_or(EscrowError::Overflow)?;
    let payout = payout_u128 as u64;

    position.claimed_winnings = true;

    // Transfer from vault; market PDA signs.
    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from:      ctx.accounts.vault.to_account_info(),
                mint:      ctx.accounts.usdc_mint.to_account_info(),
                to:        ctx.accounts.user_token_account.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[&[b"market", market_id_bytes.as_ref(), &[market_bump]]],
        ),
        payout,
        ctx.accounts.usdc_mint.decimals,
    )?;

    emit!(WinnerClaimed {
        market_id: ctx.accounts.market.market_id,
        user:      ctx.accounts.user_wallet.key(),
        amount:    payout,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimWinner<'info> {
    // Non-mut: we only read the market state and pool sizes.
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump  = market.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"position", market.key().as_ref(), user_wallet.key().as_ref()],
        bump  = user_position.bump,
    )]
    pub user_position: Account<'info, UserPosition>,

    pub user_wallet: Signer<'info>,

    #[account(
        mut,
        token::mint      = usdc_mint,
        token::authority = user_wallet,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(address = vault.mint)]
    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

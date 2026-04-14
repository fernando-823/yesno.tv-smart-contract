use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, TransferChecked};
use crate::state::{Market, MarketState, UserPosition};
use crate::errors::EscrowError;
use crate::events::VoidClaimed;

/// User claims a full refund when the market has been voided.
///
/// Both yes_stake and no_stake are returned (a user can theoretically have
/// placed bets on both sides before a void, though the UI prevents it; we
/// handle the edge case gracefully).
pub fn handler(ctx: Context<ClaimVoid>) -> Result<()> {
    require!(
        matches!(ctx.accounts.market.state, MarketState::Voided),
        EscrowError::NotVoided,
    );

    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let market_bump     = ctx.accounts.market.bump;
    let market_id       = ctx.accounts.market.market_id;

    let position = &mut ctx.accounts.user_position;
    require!(!position.void_claimed, EscrowError::VoidAlreadyClaimed);

    let refund = position.yes_stake
        .checked_add(position.no_stake)
        .ok_or(EscrowError::Overflow)?;
    require!(refund > 0, EscrowError::NoPosition);

    position.void_claimed = true;

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
        refund,
        ctx.accounts.usdc_mint.decimals,
    )?;

    emit!(VoidClaimed { market_id, user: ctx.accounts.user_wallet.key(), amount: refund });
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimVoid<'info> {
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

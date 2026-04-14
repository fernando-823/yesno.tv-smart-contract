use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, TransferChecked};
use crate::state::{ProgramConfig, Market};
use crate::errors::EscrowError;
use crate::events::PlatformClaimed;

/// Platform admin sweeps accumulated platform fees from a market vault to the
/// treasury.
///
/// `platform_accumulated` grows each time process_split_* runs.  Partial sweeps
/// are allowed — the admin can call this multiple times as more splits are
/// processed.  `platform_claimed` tracks the running total already swept so the
/// same lamports are never transferred twice.
pub fn handler(ctx: Context<ClaimPlatform>) -> Result<()> {
    let claimable = ctx.accounts.market.platform_accumulated
        .checked_sub(ctx.accounts.market.platform_claimed)
        .ok_or(EscrowError::Overflow)?;
    require!(claimable > 0, EscrowError::NoPlatformFeesToClaim);

    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let market_bump     = ctx.accounts.market.bump;
    let market_id       = ctx.accounts.market.market_id;
    let treasury_key    = ctx.accounts.platform_treasury.key();

    ctx.accounts.market.platform_claimed = ctx.accounts.market.platform_claimed
        .checked_add(claimable)
        .ok_or(EscrowError::Overflow)?;

    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from:      ctx.accounts.vault.to_account_info(),
                mint:      ctx.accounts.usdc_mint.to_account_info(),
                to:        ctx.accounts.platform_treasury.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[&[b"market", market_id_bytes.as_ref(), &[market_bump]]],
        ),
        claimable,
        ctx.accounts.usdc_mint.decimals,
    )?;

    emit!(PlatformClaimed { market_id, amount: claimable, treasury: treasury_key });
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimPlatform<'info> {
    #[account(seeds = [b"config"], bump = config.bump, has_one = admin)]
    pub config: Account<'info, ProgramConfig>,

    pub admin: Signer<'info>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump  = market.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = config.platform_treasury,
    )]
    pub platform_treasury: Account<'info, TokenAccount>,

    #[account(address = vault.mint)]
    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, TransferChecked};
use crate::state::{Market, MarketState};
use crate::errors::EscrowError;
use crate::events::CreatorClaimed;

/// The market creator claims their 5 % fee.
///
/// The creator pubkey is the one supplied at create_market time; the signer
/// of this transaction must match it.  Only callable on a Finalized market.
pub fn handler(ctx: Context<ClaimCreator>) -> Result<()> {
    require!(
        matches!(ctx.accounts.market.state, MarketState::Finalized { .. }),
        EscrowError::NotFinalized,
    );
    require!(!ctx.accounts.market.creator_fee_claimed, EscrowError::CreatorAlreadyClaimed);
    require_keys_eq!(
        ctx.accounts.creator.key(),
        ctx.accounts.market.creator,
        EscrowError::Unauthorized,
    );

    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let market_bump     = ctx.accounts.market.bump;
    let creator_owed    = ctx.accounts.market.creator_owed;
    let market_id       = ctx.accounts.market.market_id;

    ctx.accounts.market.creator_fee_claimed = true;

    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from:      ctx.accounts.vault.to_account_info(),
                mint:      ctx.accounts.usdc_mint.to_account_info(),
                to:        ctx.accounts.creator_token_account.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[&[b"market", market_id_bytes.as_ref(), &[market_bump]]],
        ),
        creator_owed,
        ctx.accounts.usdc_mint.decimals,
    )?;

    emit!(CreatorClaimed {
        market_id,
        creator: ctx.accounts.creator.key(),
        amount:  creator_owed,
    });
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimCreator<'info> {
    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump  = market.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub creator: Signer<'info>,

    #[account(
        mut,
        token::mint = usdc_mint,
    )]
    pub creator_token_account: Account<'info, TokenAccount>,

    #[account(address = vault.mint)]
    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

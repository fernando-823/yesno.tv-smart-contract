use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, TransferChecked};
use crate::state::{Market, AffiliateClaim};
use crate::errors::EscrowError;
use crate::events::AffiliateClaimedEvent;

/// Referrer claims their accumulated affiliate earnings from this market.
///
/// The referrer must be the signer.  Funds are transferred from the market
/// vault to the referrer's USDC token account.
pub fn handler(ctx: Context<ClaimAffiliate>) -> Result<()> {
    require!(!ctx.accounts.affiliate_claim.claimed, EscrowError::AffiliateAlreadyClaimed);

    let amount          = ctx.accounts.affiliate_claim.amount_owed;
    let market_id_bytes = ctx.accounts.market.market_id.to_le_bytes();
    let market_bump     = ctx.accounts.market.bump;
    let market_id       = ctx.accounts.market.market_id;
    let referrer_key    = ctx.accounts.referrer_wallet.key();

    ctx.accounts.affiliate_claim.claimed = true;

    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from:      ctx.accounts.vault.to_account_info(),
                mint:      ctx.accounts.usdc_mint.to_account_info(),
                to:        ctx.accounts.referrer_token_account.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            },
            &[&[b"market", market_id_bytes.as_ref(), &[market_bump]]],
        ),
        amount,
        ctx.accounts.usdc_mint.decimals,
    )?;

    emit!(AffiliateClaimedEvent { market_id, referrer: referrer_key, amount });
    Ok(())
}

#[derive(Accounts)]
pub struct ClaimAffiliate<'info> {
    // Non-mut: only read for signer seeds.
    pub market: Account<'info, Market>,

    #[account(
        mut,
        seeds = [b"vault", market.key().as_ref()],
        bump  = market.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds   = [b"affiliate", market.key().as_ref(), referrer_wallet.key().as_ref()],
        bump    = affiliate_claim.bump,
        has_one = market,
    )]
    pub affiliate_claim: Account<'info, AffiliateClaim>,

    pub referrer_wallet: Signer<'info>,

    #[account(
        mut,
        token::mint = usdc_mint,
    )]
    pub referrer_token_account: Account<'info, TokenAccount>,

    #[account(address = vault.mint)]
    pub usdc_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}

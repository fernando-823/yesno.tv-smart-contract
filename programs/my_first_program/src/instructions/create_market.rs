use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::{ProgramConfig, Market, MarketState};
use crate::events::MarketCreated;

/// Admin creates a market.  `market_id` uniquely identifies the market and is
/// embedded in the Market PDA seeds so every market lives at its own address.
/// `creator` is the pubkey that will receive the 5 % creator fee on settlement.
pub fn handler(ctx: Context<CreateMarket>, market_id: u64, creator: Pubkey) -> Result<()> {
    let now    = Clock::get()?.unix_timestamp;
    let market = &mut ctx.accounts.market;

    market.market_id          = market_id;
    market.creator            = creator;
    market.open_ts            = now;
    market.state              = MarketState::Open;
    market.yes_total          = 0;
    market.no_total           = 0;
    market.total_pool         = 0;
    market.prize_pool         = 0;
    market.creator_owed       = 0;
    market.platform_fee_total = 0;
    market.platform_accumulated = 0;
    market.platform_claimed   = 0;
    market.creator_fee_claimed = false;
    market.bump               = ctx.bumps.market;
    market.vault_bump         = ctx.bumps.vault;

    emit!(MarketCreated { market_id, creator, open_ts: now });
    Ok(())
}

#[derive(Accounts)]
#[instruction(market_id: u64)]
pub struct CreateMarket<'info> {
    #[account(seeds = [b"config"], bump = config.bump, has_one = admin)]
    pub config: Account<'info, ProgramConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = Market::SPACE,
        seeds = [b"market", market_id.to_le_bytes().as_ref()],
        bump,
    )]
    pub market: Account<'info, Market>,

    /// USDC token account PDA — the market PDA is the authority (vault).
    /// Vault seeds are derived from the market's key so each market gets
    /// exactly one vault.
    #[account(
        init,
        payer = admin,
        token::mint      = usdc_mint,
        token::authority = market,
        seeds = [b"vault", market.key().as_ref()],
        bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(address = config.usdc_mint)]
    pub usdc_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program:  Program<'info, Token>,
    pub rent:           Sysvar<'info, Rent>,
}

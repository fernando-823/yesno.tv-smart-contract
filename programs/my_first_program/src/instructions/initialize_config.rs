use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::ProgramConfig;

pub fn handler(ctx: Context<InitializeConfig>) -> Result<()> {
    let cfg      = &mut ctx.accounts.config;
    cfg.admin              = ctx.accounts.admin.key();
    cfg.usdc_mint          = ctx.accounts.usdc_mint.key();
    cfg.platform_treasury  = ctx.accounts.platform_treasury.key();
    cfg.bump               = ctx.bumps.config;
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
    #[account(
        init,
        payer = admin,
        space = ProgramConfig::SPACE,
        seeds = [b"config"],
        bump,
    )]
    pub config: Account<'info, ProgramConfig>,

    #[account(mut)] 
    pub admin: Signer<'info>,

    pub usdc_mint: Account<'info, Mint>,

    /// Existing USDC token account that will receive platform fees.
    #[account(token::mint = usdc_mint)]
    pub platform_treasury: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program:  Program<'info, Token>,
}

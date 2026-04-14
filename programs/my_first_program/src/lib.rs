use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod state;
pub mod instructions;

// ── Accounts struct re-exports ──────────────────────────────────────────────
// Bring every #[derive(Accounts)] struct to the crate root so the
// #[program] macro can resolve Context<T> type parameters.
pub use instructions::{
    InitializeConfig, CreateMarket, PlaceBet, SwitchSide,
    ProposeOutcome, Dispute, Finalize, VoidMarket,
    ClaimWinner, ClaimVoid, ClaimCreator,
    ProcessSplitNoRef, ProcessSplitReferred,
    ClaimAffiliate, ClaimPlatform,
};

// ── __client_accounts_* re-exports ─────────────────────────────────────────
// Anchor's #[program] macro emits `pub use crate::__client_accounts_X::*`
// inside its generated `accounts` module.  Because our Accounts structs live
// in sub-modules, the derive-generated `pub(crate) mod __client_accounts_X`
// also lives there – not at the crate root where #[program] expects it.
// Re-exporting each one here fixes the `E0432: unresolved import crate` error.
pub(crate) use instructions::initialize_config::__client_accounts_initialize_config;
pub(crate) use instructions::create_market::__client_accounts_create_market;
pub(crate) use instructions::place_bet::__client_accounts_place_bet;
pub(crate) use instructions::switch_side::__client_accounts_switch_side;
pub(crate) use instructions::propose_outcome::__client_accounts_propose_outcome;
pub(crate) use instructions::dispute::__client_accounts_dispute;
pub(crate) use instructions::finalize::__client_accounts_finalize;
pub(crate) use instructions::void_market::__client_accounts_void_market;
pub(crate) use instructions::claim_winner::__client_accounts_claim_winner;
pub(crate) use instructions::claim_void::__client_accounts_claim_void;
pub(crate) use instructions::claim_creator::__client_accounts_claim_creator;
pub(crate) use instructions::process_split_no_ref::__client_accounts_process_split_no_ref;
pub(crate) use instructions::process_split_referred::__client_accounts_process_split_referred;
pub(crate) use instructions::claim_affiliate::__client_accounts_claim_affiliate;
pub(crate) use instructions::claim_platform::__client_accounts_claim_platform;

declare_id!("BZ686ej4wmm7rV3fdGQtJVevrFndQJsDMYJQmU3Mg7d1");

#[program]
pub mod yesno_escrow {
    use super::*;

    // ── Admin ─────────────────────────────────────────────────────────────
    pub fn initialize_config(ctx: Context<InitializeConfig>) -> Result<()> {
        instructions::initialize_config::handler(ctx)
    }

    // ── Market creation ───────────────────────────────────────────────────
    pub fn create_market(
        ctx: Context<CreateMarket>,
        market_id: u64,
        creator: Pubkey,
    ) -> Result<()> {
        instructions::create_market::handler(ctx, market_id, creator)
    }

    // ── Betting phase ─────────────────────────────────────────────────────
    pub fn place_bet(
        ctx: Context<PlaceBet>,  
        market_id: u64,
        side: bool,
        amount: u64,
        referrer: Option<Pubkey>,
    ) -> Result<()> {
        instructions::place_bet::handler(ctx, market_id, side, amount, referrer)
    }

    pub fn switch_side(ctx: Context<SwitchSide>) -> Result<()> {
        instructions::switch_side::handler(ctx)
    }

    // ── Resolution ────────────────────────────────────────────────────────
    pub fn propose_outcome(
        ctx: Context<ProposeOutcome>,
        winning_side: bool,
    ) -> Result<()> {
        instructions::propose_outcome::handler(ctx, winning_side)
    }

    pub fn dispute(ctx: Context<Dispute>) -> Result<()> {
        instructions::dispute::handler(ctx)
    }

    pub fn finalize(ctx: Context<Finalize>, winning_side: bool) -> Result<()> {
        instructions::finalize::handler(ctx, winning_side)
    }

    pub fn void_market(ctx: Context<VoidMarket>) -> Result<()> {
        instructions::void_market::handler(ctx)
    }

    // ── Claims ────────────────────────────────────────────────────────────
    pub fn claim_winner(ctx: Context<ClaimWinner>) -> Result<()> {
        instructions::claim_winner::handler(ctx)
    }

    pub fn claim_void(ctx: Context<ClaimVoid>) -> Result<()> {
        instructions::claim_void::handler(ctx)
    }

    pub fn claim_creator(ctx: Context<ClaimCreator>) -> Result<()> {
        instructions::claim_creator::handler(ctx)
    }

    // ── Lazy fee attribution ──────────────────────────────────────────────
    pub fn process_split_no_ref(ctx: Context<ProcessSplitNoRef>) -> Result<()> {
        instructions::process_split_no_ref::handler(ctx)
    }

    pub fn process_split_referred(ctx: Context<ProcessSplitReferred>) -> Result<()> {
        instructions::process_split_referred::handler(ctx)
    }

    pub fn claim_affiliate(ctx: Context<ClaimAffiliate>) -> Result<()> {
        instructions::claim_affiliate::handler(ctx)
    }

    pub fn claim_platform(ctx: Context<ClaimPlatform>) -> Result<()> {
        instructions::claim_platform::handler(ctx)
    }
}

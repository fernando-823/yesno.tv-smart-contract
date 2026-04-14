use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Betting window has closed (first 15 s only)")]
    BettingWindowClosed,
    #[msg("Switching window is not open yet (starts at 15 s)")]
    SwitchingWindowNotOpen,
    #[msg("Switching window has closed (30 s+ is locked)")]
    SwitchingWindowClosed,
    #[msg("Market pools are not locked yet — 30 seconds must elapse")]
    MarketNotLocked,
    #[msg("Market is not in the expected state for this action")]
    InvalidMarketState,
    #[msg("Dispute window has already expired (3 minutes)")]
    DisputeWindowExpired,
    #[msg("Dispute window has not expired yet — wait 3 minutes after outcome proposal")]
    DisputeWindowNotExpired,
    #[msg("Only the program admin can perform this action")]
    Unauthorized,
    #[msg("Amount must be greater than zero")]
    ZeroAmount,
    #[msg("User has no stake on the winning side")]
    NotAWinner,
    #[msg("Winnings already claimed")]
    AlreadyClaimed,
    #[msg("Market was not voided — cannot refund")]
    NotVoided,
    #[msg("Void refund already claimed for this position")]
    VoidAlreadyClaimed,
    #[msg("Referrer cannot be the user themselves")]
    SelfReferral,
    #[msg("Referrer does not match the stored referrer for this user")]
    InvalidReferrer,
    #[msg("Platform split already processed for this position")]
    SplitAlreadyDone,
    #[msg("Affiliate claim already made for this market")]
    AffiliateAlreadyClaimed,
    #[msg("User has no stake to switch — place a bet first")]
    NoStakeToSwitch,
    #[msg("Creator fee already claimed")]
    CreatorAlreadyClaimed,
    #[msg("User already has stake on the other side — switch instead of placing a new bet")]
    AlreadyOnOtherSide,
    #[msg("Arithmetic overflow in fee calculation")]
    Overflow,
    #[msg("No platform fees accumulated to claim yet")]
    NoPlatformFeesToClaim,
    #[msg("Market is not finalised — cannot claim winner payout")]
    NotFinalized,
    #[msg("User has no stake in this market")]
    NoPosition,
    #[msg("Winning side has zero stakers — no prize pool to distribute")]
    NoWinningSide,
    #[msg("process_split_referred must be used for users with a referrer")]
    UserHasReferrer,
    #[msg("process_split_no_ref must only be used for users without a referrer")]
    UserHasNoReferrer,
}

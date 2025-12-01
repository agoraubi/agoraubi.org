//! # AGORA Governance v1.0
//! 
//! # ╔════════════════════════════════════════════════════════════════════════════════════════════════╗
//! # ║                                                                                                ║
//! # ║   ██╗   ██╗██████╗  ██████╗ ██████╗  █████╗ ██████╗ ███████╗ █████╗ ██████╗ ██╗     ███████╗   ║
//! # ║   ██║   ██║██╔══██╗██╔════╝ ██╔══██╗██╔══██╗██╔══██╗██╔════╝██╔══██╗██╔══██╗██║     ██╔════╝   ║
//! # ║   ██║   ██║██████╔╝██║  ███╗██████╔╝███████║██║  ██║█████╗  ███████║██████╔╝██║     █████╗     ║
//! # ║   ██║   ██║██╔═══╝ ██║   ██║██╔══██╗██╔══██║██║  ██║██╔══╝  ██╔══██║██╔══██╗██║     ██╔══╝     ║
//! # ║   ╚██████╔╝██║     ╚██████╔╝██║  ██║██║  ██║██████╔╝███████╗██║  ██║██████╔╝███████╗███████╗   ║
//! # ║    ╚═════╝ ╚═╝      ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚═╝  ╚═╝╚═════╝ ╚══════╝╚══════╝   ║
//! # ║                                                                                                ║
//! # ║   This program is UPGRADEABLE by DAO vote.                                                     ║
//! # ║   Constitutional proposals (67% supermajority) can modify this code.                           ║
//! # ║                                                                                                ║
//! # ╚════════════════════════════════════════════════════════════════════════════════════════════════╝
//!
//! ## Purpose
//! 
//! AGORA Governance handles all the changeable, democratic parts of the protocol:
//! 
//! - **DAO Proposals** - Create, vote, finalize proposals
//! - **Country Sanctions** - Impose/lift sanctions via supermajority vote
//! - **Fee Parameters** - Adjust transfer fees (base rate, burn percentage)
//! - **Merchant Detection** - Volume thresholds for merchant tiers
//! - **Gas Pool** - SOL subsidies for new users, sponsor system
//! - **Treasury** - Protocol funds controlled by DAO
//! 
//! ## What This Program Does NOT Handle
//! 
//! The sacred, immutable parts are in `agora-core`:
//! 
//! - 100 AGORA/day (IMMUTABLE - cannot be changed by anyone)
//! - User registration and biometric deduplication
//! - Basic token minting and transfers
//! 
//! ## Relationship with Core Program
//! 
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                        AGORA CORE (IMMUTABLE)                           │
//! │                                                                         │
//! │   • 100 AGORA/day - forever                                             │
//! │   • Reads sanction accounts from this program                           │
//! │   • Reads fee parameters from this program                              │
//! │   • We CANNOT modify Core's mint amount                                 │
//! │   • We can ONLY create accounts that Core reads                         │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//!                                    ▲
//!                                    │ creates accounts that Core reads
//!                                    │
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                     AGORA GOVERNANCE (THIS PROGRAM)                     │
//! │                                                                         │
//! │   • DAO proposals and voting                                            │
//! │   • Country sanctions (creates CountrySanction accounts)                │
//! │   • Fee parameters (creates FeeState account)                           │
//! │   • Gas pool and sponsors                                               │
//! │   • Merchant detection and tiers                                        │
//! │   • Treasury management                                                 │
//! │                                                                         │
//! │   Upgradeable by Constitutional proposal (67% supermajority)            │
//! │                                                                         │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Governance Model
//! 
//! - **1 person = 1 vote** (not token-weighted!)
//! - Proposals require bonds to prevent spam
//! - Reputation system rewards good proposers, punishes spammers
//! - Different proposal types have different requirements
//!
//! ## License
//! 
//! Apache 2.0 - See LICENSE file

use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_instruction;

declare_id!("AGoRAGovXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    SECTION 1: CONSTANTS                                   ║
// ║                                                                           ║
// ║   All parameters that the DAO can adjust are defined here.                ║
// ║   These are the INITIAL values - DAO can change them via proposals.       ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    GOVERNANCE CHANGE REQUIREMENTS                         ║
// ║                                                                           ║
// ║   Each constant/function in this program has a required vote threshold    ║
// ║   to change it. These are marked with the following tags:                 ║
// ║                                                                           ║
// ║   🟢 STANDARD (>50%)        - Simple majority, 1% users, 3 days           ║
// ║   🟡 TREASURY (>50%)        - Simple majority, 2% users, 7 days           ║
// ║   🟠 SANCTION (>67%)        - Supermajority, 5% users, 14 days            ║
// ║   🔴 CONSTITUTIONAL (>75%)  - High supermajority, 10% users, 14 days      ║
// ║   ⚫ IMMUTABLE              - Cannot be changed (in AGORA Core)           ║
// ║                                                                           ║
// ║   Quorums are DYNAMIC - calculated as % of total registered users.        ║
// ║   See Section 1.3 for min/max bounds.                                     ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

// ============================================================================
// 1.1 FEE PARAMETERS
// ============================================================================
// 🔴 CONSTITUTIONAL (>75%) - All fee parameters require supermajority to change

/// Base fee rate in basis points (0.05% = 5 basis points)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const DEFAULT_BASE_FEE_RATE: u64 = 5;

/// Fee divisor for basis point calculation
pub const FEE_DIVISOR: u64 = 10000;

/// Default percentage of fees that are burned (50%)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const DEFAULT_BURN_PERCENTAGE: u64 = 50;

// Activity-based fee multipliers (in basis points, 100 = 1x)
// 🟢 STANDARD (>50%) to change these multipliers
pub const ACTIVE_MULTIPLIER: u64 = 80;    // 0.8x for active users
pub const NORMAL_MULTIPLIER: u64 = 100;   // 1.0x normal
pub const INACTIVE_MULTIPLIER: u64 = 150; // 1.5x for inactive
pub const DORMANT_MULTIPLIER: u64 = 200;  // 2.0x for dormant

// Activity thresholds (in seconds)
// 🟢 STANDARD (>50%) to change these thresholds
pub const ACTIVE_THRESHOLD: i64 = 7 * 86400;    // 7 days
pub const NORMAL_THRESHOLD: i64 = 30 * 86400;   // 30 days
pub const INACTIVE_THRESHOLD: i64 = 90 * 86400; // 90 days

// ============================================================================
// 1.2 PROPOSAL BONDS
// ============================================================================
// 
// High bonds prevent spam attacks. These amounts require pooling resources
// from multiple community members, ensuring proposals have real backing.
// 🔴 CONSTITUTIONAL (>75%) to change bond amounts

/// Standard proposal bond (20,000 AGORA)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const PROPOSAL_BOND_STANDARD: u64 = 20_000_000_000_000;

/// Treasury proposal bond (50,000 AGORA)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const PROPOSAL_BOND_TREASURY: u64 = 50_000_000_000_000;

/// Constitutional proposal bond (100,000 AGORA) - highest tier
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const PROPOSAL_BOND_CONSTITUTIONAL: u64 = 100_000_000_000_000;

/// Sanction proposal bond (75,000 AGORA)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const PROPOSAL_BOND_SANCTION: u64 = 75_000_000_000_000;

// ============================================================================
// 1.3 DYNAMIC QUORUM SYSTEM
// ============================================================================
// 
// Quorum is calculated as percentage of total registered users.
// This ensures governance scales with protocol growth.
// 
// Formula: quorum = max(MINIMUM, total_users * PERCENTAGE / 10000)
// 
// Minimums are set to 1M user equivalent - no governance changes until
// protocol is established with at least 1 million users.
// No maximum cap - democracy scales infinitely.
// 
// 🔴 CONSTITUTIONAL (>75%) to change quorum requirements

// Quorum percentages (in basis points, 100 = 1%)
/// Standard proposal: 1% of users
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const QUORUM_PCT_STANDARD: u64 = 100;       // 1%

/// Treasury proposal: 2% of users
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const QUORUM_PCT_TREASURY: u64 = 200;       // 2%

/// Sanction proposal: 5% of users
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const QUORUM_PCT_SANCTION: u64 = 500;       // 5%

/// Constitutional proposal: 10% of users
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const QUORUM_PCT_CONSTITUTIONAL: u64 = 1000; // 10%

// Minimum quorums (equivalent to 1M users - no changes until protocol is established)
pub const QUORUM_MIN_STANDARD: u64 = 10_000;       // 1% of 1M
pub const QUORUM_MIN_TREASURY: u64 = 20_000;       // 2% of 1M
pub const QUORUM_MIN_SANCTION: u64 = 50_000;       // 5% of 1M
pub const QUORUM_MIN_CONSTITUTIONAL: u64 = 100_000; // 10% of 1M

// ============================================================================
// 1.4 APPROVAL THRESHOLDS (in basis points, 5000 = 50%)
// ============================================================================
// 🔴 CONSTITUTIONAL (>75%) to change approval thresholds
// WARNING: Changing these affects the fundamental governance structure!

/// Standard proposal approval (>50%)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const APPROVAL_STANDARD: u64 = 5001;

/// Treasury proposal approval (>50%)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const APPROVAL_TREASURY: u64 = 5001;

/// Sanction proposal approval (>67%)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const APPROVAL_SANCTION: u64 = 6700;

/// Constitutional proposal approval (>75%)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const APPROVAL_CONSTITUTIONAL: u64 = 7500;

// ============================================================================
// 1.5 VOTING PERIODS (in seconds)
// ============================================================================
// 🔴 CONSTITUTIONAL (>75%) to change voting periods

/// Standard proposal voting period (3 days)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const VOTING_PERIOD_STANDARD: i64 = 3 * 86400;

/// Treasury proposal voting period (7 days)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const VOTING_PERIOD_TREASURY: i64 = 7 * 86400;

/// Constitutional proposal voting period (14 days)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const VOTING_PERIOD_CONSTITUTIONAL: i64 = 14 * 86400;

/// Sanction proposal voting period (14 days)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const VOTING_PERIOD_SANCTION: i64 = 14 * 86400;

// ============================================================================
// 1.6 REPUTATION SYSTEM
// ============================================================================
//
// Reputation is calculated AUTOMATICALLY from voting results.
// No manual flagging - this prevents coordinated attacks on legitimate proposers.
// 🔴 CONSTITUTIONAL (>75%) to change reputation parameters

/// Reputation gain when proposal passes
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const REP_PROPOSAL_PASSED: i32 = 2;

/// Reputation gain when proposal rejected but reached quorum
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const REP_PROPOSAL_REJECTED: i32 = 1;

/// Reputation loss when proposal reaches 50%+ of quorum but expires
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const REP_NO_QUORUM_50: i32 = -1;

/// Reputation loss when proposal reaches 25-50% of quorum
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const REP_NO_QUORUM_25: i32 = -2;

/// Reputation loss when proposal reaches <25% of quorum (spam)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const REP_NO_QUORUM_10: i32 = -3;

/// Threshold percentages for quorum (in basis points)
pub const QUORUM_THRESHOLD_50: u64 = 5000;  // 50%
pub const QUORUM_THRESHOLD_25: u64 = 2500;  // 25%

/// Reputation threshold for ban (-10)
/// 🔴 CONSTITUTIONAL (>75%) to change
pub const REP_THRESHOLD_BAN: i32 = -10;

// ============================================================================
// 1.7 MERCHANT DETECTION THRESHOLDS
// ============================================================================
//
// Merchants are detected automatically based on transaction patterns.
// Two paths to qualify: many unique customers OR high volume.
// 🟢 STANDARD (>50%) to change merchant thresholds

// Unique customer thresholds - 🟢 STANDARD (>50%) to change
pub const EMERGING_MIN_CUSTOMERS: u32 = 10;
pub const SMALL_MIN_CUSTOMERS: u32 = 25;
pub const MEDIUM_MIN_CUSTOMERS: u32 = 100;
pub const LARGE_MIN_CUSTOMERS: u32 = 500;
pub const ENTERPRISE_MIN_CUSTOMERS: u32 = 2000;

// Monthly volume thresholds (in AGORA base units) - 🟢 STANDARD (>50%) to change
pub const EMERGING_MIN_VOLUME: u128 = 1_000_000_000_000;      // 1,000 AGORA
pub const SMALL_MIN_VOLUME: u128 = 10_000_000_000_000;        // 10,000 AGORA
pub const MEDIUM_MIN_VOLUME: u128 = 50_000_000_000_000;       // 50,000 AGORA
pub const LARGE_MIN_VOLUME: u128 = 100_000_000_000_000;       // 100,000 AGORA
pub const ENTERPRISE_MIN_VOLUME: u128 = 500_000_000_000_000;  // 500,000 AGORA

// Merchant fee discounts (in basis points, 100 = no discount) - 🟢 STANDARD (>50%) to change
pub const EMERGING_FEE_DISCOUNT: u64 = 75;   // 25% off
pub const SMALL_FEE_DISCOUNT: u64 = 50;      // 50% off
pub const MEDIUM_FEE_DISCOUNT: u64 = 25;     // 75% off
pub const LARGE_FEE_DISCOUNT: u64 = 0;       // FREE
pub const ENTERPRISE_FEE_DISCOUNT: u64 = 0;  // FREE

// ============================================================================
// 1.8 GAS POOL / SPONSOR THRESHOLDS
// ============================================================================
// 🟢 STANDARD (>50%) to change gas pool parameters

// Sponsor tier thresholds (in lamports) - 🟢 STANDARD (>50%) to change
pub const BRONZE_THRESHOLD: u64 = 1_000_000_000;      // 1 SOL
pub const SILVER_THRESHOLD: u64 = 10_000_000_000;     // 10 SOL
pub const GOLD_THRESHOLD: u64 = 100_000_000_000;      // 100 SOL
pub const PLATINUM_THRESHOLD: u64 = 1_000_000_000_000; // 1,000 SOL
pub const DIAMOND_THRESHOLD: u64 = 10_000_000_000_000; // 10,000 SOL

// Personal allocation percentages (% of contribution for personal use) - 🟢 STANDARD (>50%) to change
pub const BRONZE_PERSONAL_PCT: u64 = 10;
pub const SILVER_PERSONAL_PCT: u64 = 15;
pub const GOLD_PERSONAL_PCT: u64 = 20;
pub const PLATINUM_PERSONAL_PCT: u64 = 25;
pub const DIAMOND_PERSONAL_PCT: u64 = 30;

// Sponsor fee discounts (in basis points) - 🟢 STANDARD (>50%) to change
pub const BRONZE_SPONSOR_DISCOUNT: u64 = 90;   // 10% off
pub const SILVER_SPONSOR_DISCOUNT: u64 = 80;   // 20% off
pub const GOLD_SPONSOR_DISCOUNT: u64 = 60;     // 40% off
pub const PLATINUM_SPONSOR_DISCOUNT: u64 = 40; // 60% off
pub const DIAMOND_SPONSOR_DISCOUNT: u64 = 0;   // FREE

// Gas pool limits
pub const FREE_TIER_DAILY_TX: u8 = 5;          // Max 5 subsidized TX/day
pub const MAX_SUBSIDY_PER_TX: u64 = 10_000;    // Max 0.00001 SOL per TX

// ============================================================================
// 1.9 TREASURY MINT RATE
// ============================================================================
//
// Treasury receives a small mint for each user claim.
// Rate: 1/(π×e) ≈ 0.117% - a mathematical constant, not arbitrary.
// 🔴 CONSTITUTIONAL (>75%) to change treasury mint rate

pub const TREASURY_MINT_NUMERATOR: u64 = 1000;
pub const TREASURY_MINT_DENOMINATOR: u64 = 854513; // Approximates 1/(π×e)

// ============================================================================
// 1.10 SANCTION PARAMETERS
// ============================================================================
// 🟠 SANCTION (>67%) to change sanction parameters
// These protect against abuse of the sanction system itself

/// Minimum UBI percentage for sanctions (never zero)
/// 🟠 SANCTION (>67%) to change - protects against complete exclusion
pub const MIN_SANCTION_UBI_PCT: u8 = 1;

/// Maximum sanction duration (365 days)
/// 🟠 SANCTION (>67%) to change
pub const MAX_SANCTION_DURATION: i64 = 365 * 86400;

/// Default sanction duration (365 days)
/// 🟠 SANCTION (>67%) to change
pub const DEFAULT_SANCTION_DURATION: i64 = 365 * 86400;

// ============================================================================
// 1.11 HELPER FUNCTIONS FOR DYNAMIC QUORUM
// ============================================================================

/// Calculate dynamic quorum based on total users
/// 
/// Formula: max(MINIMUM, min(MAXIMUM, total_users * PERCENTAGE / 10000))
/// 
/// # Arguments
/// * `total_users` - Total registered users in protocol
/// * `proposal_type` - Type of proposal
/// 
/// # Returns
/// * Calculated quorum (number of votes required)
pub fn calculate_quorum(total_users: u64, proposal_type: &ProposalType) -> u64 {
    let (pct, min) = match proposal_type {
        ProposalType::Standard => (QUORUM_PCT_STANDARD, QUORUM_MIN_STANDARD),
        ProposalType::Treasury => (QUORUM_PCT_TREASURY, QUORUM_MIN_TREASURY),
        ProposalType::Sanction => (QUORUM_PCT_SANCTION, QUORUM_MIN_SANCTION),
        ProposalType::Constitutional => (QUORUM_PCT_CONSTITUTIONAL, QUORUM_MIN_CONSTITUTIONAL),
    };
    
    // Calculate: total_users * pct / 10000 (pct is in basis points)
    let calculated = total_users.saturating_mul(pct) / 10000;
    
    // Apply minimum only (no maximum - democracy scales infinitely)
    calculated.max(min)
}

/// Get approval threshold for proposal type
pub fn get_approval_threshold(proposal_type: &ProposalType) -> u64 {
    match proposal_type {
        ProposalType::Standard => APPROVAL_STANDARD,
        ProposalType::Treasury => APPROVAL_TREASURY,
        ProposalType::Sanction => APPROVAL_SANCTION,
        ProposalType::Constitutional => APPROVAL_CONSTITUTIONAL,
    }
}

/// Get voting period for proposal type
pub fn get_voting_period(proposal_type: &ProposalType) -> i64 {
    match proposal_type {
        ProposalType::Standard => VOTING_PERIOD_STANDARD,
        ProposalType::Treasury => VOTING_PERIOD_TREASURY,
        ProposalType::Sanction => VOTING_PERIOD_SANCTION,
        ProposalType::Constitutional => VOTING_PERIOD_CONSTITUTIONAL,
    }
}

/// Get bond amount for proposal type
pub fn get_bond_amount(proposal_type: &ProposalType) -> u64 {
    match proposal_type {
        ProposalType::Standard => PROPOSAL_BOND_STANDARD,
        ProposalType::Treasury => PROPOSAL_BOND_TREASURY,
        ProposalType::Sanction => PROPOSAL_BOND_SANCTION,
        ProposalType::Constitutional => PROPOSAL_BOND_CONSTITUTIONAL,
    }
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    SECTION 2: PROGRAM ENTRY POINT                         ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

#[program]
pub mod agora_governance {
    use super::*;

    // ========================================================================
    // 2.1 INITIALIZATION
    // ========================================================================

    /// Initialize the Governance program.
    /// 
    /// Sets up:
    /// - Governance state PDA
    /// - Fee state with default values
    /// - Proposal registry
    /// - Gas pool state
    pub fn initialize(ctx: Context<InitializeGovernance>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        
        // Initialize governance state
        let gov_state = &mut ctx.accounts.governance_state;
        gov_state.authority = ctx.accounts.authority.key();
        gov_state.core_program = ctx.accounts.core_program.key();
        gov_state.is_initialized = true;
        gov_state.launch_timestamp = current_time;
        gov_state.bump = ctx.bumps.governance_state;
        
        // Initialize fee state with defaults
        let fee_state = &mut ctx.accounts.fee_state;
        fee_state.base_fee_rate = DEFAULT_BASE_FEE_RATE;
        fee_state.burn_percentage = DEFAULT_BURN_PERCENTAGE;
        fee_state.active_multiplier = ACTIVE_MULTIPLIER;
        fee_state.normal_multiplier = NORMAL_MULTIPLIER;
        fee_state.inactive_multiplier = INACTIVE_MULTIPLIER;
        fee_state.dormant_multiplier = DORMANT_MULTIPLIER;
        fee_state.last_updated = current_time;
        fee_state.bump = ctx.bumps.fee_state;
        
        // Initialize proposal registry
        let registry = &mut ctx.accounts.proposal_registry;
        registry.next_proposal_id = 1;
        registry.total_proposals = 0;
        registry.active_proposals = 0;
        registry.total_bonds_forfeited = 0;
        registry.total_bonds_returned = 0;
        registry.bump = ctx.bumps.proposal_registry;
        
        emit!(GovernanceInitialized {
            authority: gov_state.authority,
            core_program: gov_state.core_program,
            timestamp: current_time,
        });
        
        msg!("AGORA Governance initialized.");
        
        Ok(())
    }

    // ========================================================================
    // 2.2 DAO PROPOSALS
    // ========================================================================

    /// Create a new proposal.
    /// 
    /// # Bond Mechanism
    /// 
    /// Proposer must stake a bond (20K-100K AGORA depending on type).
    /// Bond is returned if proposal reaches 50% of quorum.
    /// Bond is forfeited if proposal fails to attract interest.
    /// 
    /// # Reputation
    /// 
    /// Proposer's reputation affects bond cost:
    /// - bond_multiplier = 1 + (abs(reputation) / 2)
    /// - At -10 reputation: banned from creating proposals
    /// 
    /// # Arguments
    /// * `proposal_type` - Type of proposal (Standard, Treasury, Constitutional, Sanction)
    /// * `title` - Short title (64 bytes max)
    /// * `description_hash` - IPFS hash of full description
    /// * `treasury_amount` - Amount requested (for Treasury proposals)
    /// * `treasury_recipient` - Recipient (for Treasury proposals)
    /// * `sanction_country` - Country code (for Sanction proposals)
    /// * `sanction_ubi_pct` - UBI percentage (for Sanction proposals)
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_type: ProposalType,
        title: [u8; 64],
        description_hash: [u8; 32],
        treasury_amount: u64,
        treasury_recipient: Pubkey,
        sanction_country: [u8; 3],
        sanction_ubi_pct: u8,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let proposer_state = &mut ctx.accounts.proposer_state;
        let registry = &mut ctx.accounts.proposal_registry;
        let proposal = &mut ctx.accounts.proposal;
        
        // ====================================================================
        // CHECK REPUTATION (banned at -10)
        // ====================================================================
        
        require!(
            proposer_state.proposal_reputation > REP_THRESHOLD_BAN,
            GovernanceError::ProposerBanned
        );
        
        // ====================================================================
        // CALCULATE BOND WITH MULTIPLIER
        // ====================================================================
        
        let base_bond = get_bond_amount(&proposal_type);
        
        // Bond multiplier: 1 + (abs(reputation) / 2)
        let bond_multiplier: u64 = if proposer_state.proposal_reputation >= 0 {
            1
        } else {
            1 + (proposer_state.proposal_reputation.unsigned_abs() / 2) as u64
        };
        
        let required_bond = base_bond.saturating_mul(bond_multiplier);
        
        // ====================================================================
        // GET DYNAMIC QUORUM, APPROVAL, VOTING PERIOD
        // ====================================================================
        
        // Read total users from Core protocol state
        // We deserialize only the total_users field (offset 32+32+32+8 = 104 bytes)
        let core_data = ctx.accounts.core_protocol_state.try_borrow_data()?;
        // ProtocolState layout: authority(32) + mint(32) + treasury(32) + total_users(8)
        let total_users = if core_data.len() >= 112 {
            u64::from_le_bytes(core_data[104..112].try_into().unwrap_or([0u8; 8]))
        } else {
            QUORUM_MIN_CONSTITUTIONAL  // Fallback to minimum if can't read
        };
        
        // Calculate dynamic quorum based on total users
        let quorum = calculate_quorum(total_users, &proposal_type);
        let approval_threshold = get_approval_threshold(&proposal_type);
        let voting_period = get_voting_period(&proposal_type);
        
        // ====================================================================
        // VALIDATE SANCTION PARAMETERS
        // ====================================================================
        
        if let ProposalType::Sanction = proposal_type {
            require!(
                sanction_ubi_pct >= MIN_SANCTION_UBI_PCT && sanction_ubi_pct <= 99,
                GovernanceError::InvalidSanctionPercentage
            );
        }
        
        // ====================================================================
        // TRANSFER BOND
        // ====================================================================
        
        // Transfer AGORA tokens as bond
        let cpi_accounts = anchor_spl::token::Transfer {
            from: ctx.accounts.proposer_token_account.to_account_info(),
            to: ctx.accounts.bond_escrow.to_account_info(),
            authority: ctx.accounts.proposer.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
        );
        anchor_spl::token::transfer(cpi_ctx, required_bond)?;
        
        // ====================================================================
        // CREATE PROPOSAL
        // ====================================================================
        
        proposal.id = registry.next_proposal_id;
        proposal.proposer = ctx.accounts.proposer.key();
        proposal.proposal_type = proposal_type.clone();
        proposal.status = ProposalStatus::Active;
        proposal.title = title;
        proposal.description_hash = description_hash;
        proposal.bond_amount = required_bond;
        proposal.bond_resolved = false;
        proposal.votes_yes = 0;
        proposal.votes_no = 0;
        proposal.votes_abstain = 0;
        proposal.total_voters = 0;
        proposal.quorum_required = quorum;
        proposal.approval_threshold = approval_threshold;
        proposal.created_at = current_time;
        proposal.voting_ends_at = current_time + voting_period;
        proposal.executed_at = 0;
        proposal.treasury_amount = treasury_amount;
        proposal.treasury_recipient = treasury_recipient;
        proposal.sanction_country = sanction_country;
        proposal.sanction_ubi_pct = sanction_ubi_pct;
        proposal.bump = ctx.bumps.proposal;
        
        // Update registry
        registry.next_proposal_id += 1;
        registry.total_proposals += 1;
        registry.active_proposals += 1;
        
        // Update proposer stats
        proposer_state.proposals_created += 1;
        
        emit!(ProposalCreated {
            id: proposal.id,
            proposer: proposal.proposer,
            proposal_type,
            title,
            bond_amount: required_bond,
            quorum_required: quorum,
            voting_ends_at: proposal.voting_ends_at,
            timestamp: current_time,
        });
        
        msg!("Proposal #{} created. Bond: {} AGORA", proposal.id, required_bond / 1_000_000_000);
        
        Ok(())
    }

    /// Vote on an active proposal.
    /// 
    /// # Voting Rules
    /// 
    /// - 1 person = 1 vote (not token-weighted)
    /// - Can vote Yes, No, or Abstain
    /// - Cannot change vote once cast
    /// - Must be verified user
    /// 
    /// # Arguments
    /// * `choice` - 0=No, 1=Yes, 2=Abstain
    pub fn vote_on_proposal(
        ctx: Context<VoteOnProposal>,
        choice: u8,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let proposal = &mut ctx.accounts.proposal;
        let vote_record = &mut ctx.accounts.vote_record;
        let voter_state = &ctx.accounts.voter_state;
        
        // ====================================================================
        // VALIDATION
        // ====================================================================
        
        require!(voter_state.is_verified, GovernanceError::VoterNotVerified);
        require!(proposal.status == ProposalStatus::Active, GovernanceError::ProposalNotActive);
        require!(current_time <= proposal.voting_ends_at, GovernanceError::VotingEnded);
        require!(choice <= 2, GovernanceError::InvalidVoteChoice);
        require!(!vote_record.has_voted, GovernanceError::AlreadyVoted);
        
        // ====================================================================
        // RECORD VOTE
        // ====================================================================
        
        vote_record.voter = ctx.accounts.voter.key();
        vote_record.proposal_id = proposal.id;
        vote_record.choice = choice;
        vote_record.voted_at = current_time;
        vote_record.has_voted = true;
        vote_record.bump = ctx.bumps.vote_record;
        
        // Update proposal tallies
        match choice {
            0 => proposal.votes_no += 1,
            1 => proposal.votes_yes += 1,
            2 => proposal.votes_abstain += 1,
            _ => return Err(GovernanceError::InvalidVoteChoice.into()),
        }
        proposal.total_voters += 1;
        
        emit!(VoteCast {
            proposal_id: proposal.id,
            voter: vote_record.voter,
            choice,
            timestamp: current_time,
        });
        
        Ok(())
    }

    /// Finalize a proposal after voting ends.
    /// 
    /// # Outcomes
    /// 
    /// 1. **Passed**: Quorum reached + approval threshold met
    ///    - Proposer gains +2 reputation
    ///    - Bond returned
    ///    - Proposal executed (if applicable)
    /// 
    /// 2. **Rejected**: Quorum reached + approval threshold NOT met
    ///    - Proposer gains +1 reputation (legitimate but unpopular)
    ///    - Bond returned
    /// 
    /// 3. **Expired (50%+ quorum)**: Close but not enough interest
    ///    - Proposer loses -1 reputation
    ///    - Bond forfeited
    /// 
    /// 4. **Expired (25-50% quorum)**: Low interest
    ///    - Proposer loses -2 reputation
    ///    - Bond forfeited
    /// 
    /// 5. **Expired (<25% quorum)**: Spam/irrelevant
    ///    - Proposer loses -3 reputation
    ///    - Bond forfeited
    pub fn finalize_proposal(ctx: Context<FinalizeProposal>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let proposal = &mut ctx.accounts.proposal;
        let proposer_state = &mut ctx.accounts.proposer_state;
        let registry = &mut ctx.accounts.proposal_registry;
        
        // ====================================================================
        // VALIDATION
        // ====================================================================
        
        require!(proposal.status == ProposalStatus::Active, GovernanceError::ProposalNotActive);
        require!(current_time > proposal.voting_ends_at, GovernanceError::VotingNotEnded);
        
        // ====================================================================
        // CALCULATE RESULTS
        // ====================================================================
        
        let total_votes = proposal.votes_yes + proposal.votes_no;
        let quorum_reached = total_votes >= proposal.quorum_required;
        
        let approval_pct = if total_votes > 0 {
            (proposal.votes_yes * 10000) / total_votes
        } else {
            0
        };
        
        let approved = approval_pct >= proposal.approval_threshold;
        
        // ====================================================================
        // DETERMINE OUTCOME AND REPUTATION CHANGE
        // ====================================================================
        
        let (new_status, reputation_change, return_bond) = if quorum_reached {
            if approved {
                (ProposalStatus::Passed, REP_PROPOSAL_PASSED, true)
            } else {
                (ProposalStatus::Rejected, REP_PROPOSAL_REJECTED, true)
            }
        } else {
            // Calculate quorum percentage
            let quorum_pct = (total_votes * 10000) / proposal.quorum_required;
            
            if quorum_pct >= QUORUM_THRESHOLD_50 {
                (ProposalStatus::Expired, REP_NO_QUORUM_50, false)
            } else if quorum_pct >= QUORUM_THRESHOLD_25 {
                (ProposalStatus::Expired, REP_NO_QUORUM_25, false)
            } else {
                (ProposalStatus::Expired, REP_NO_QUORUM_10, false)
            }
        };
        
        // ====================================================================
        // UPDATE PROPOSAL STATUS
        // ====================================================================
        
        proposal.status = new_status.clone();
        proposal.executed_at = current_time;
        proposal.bond_resolved = true;
        
        // ====================================================================
        // UPDATE PROPOSER REPUTATION
        // ====================================================================
        
        proposer_state.proposal_reputation = proposer_state
            .proposal_reputation
            .saturating_add(reputation_change);
        
        // ====================================================================
        // HANDLE BOND
        // ====================================================================
        
        if return_bond {
            // Return bond to proposer
            let seeds = &[
                b"governance",
                &[ctx.accounts.governance_state.bump],
            ];
            let signer_seeds = &[&seeds[..]];
            
            let cpi_accounts = anchor_spl::token::Transfer {
                from: ctx.accounts.bond_escrow.to_account_info(),
                to: ctx.accounts.proposer_token_account.to_account_info(),
                authority: ctx.accounts.governance_state.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            anchor_spl::token::transfer(cpi_ctx, proposal.bond_amount)?;
            
            registry.total_bonds_returned += proposal.bond_amount;
        } else {
            // Forfeit bond to treasury
            let seeds = &[
                b"governance",
                &[ctx.accounts.governance_state.bump],
            ];
            let signer_seeds = &[&seeds[..]];
            
            let cpi_accounts = anchor_spl::token::Transfer {
                from: ctx.accounts.bond_escrow.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: ctx.accounts.governance_state.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            anchor_spl::token::transfer(cpi_ctx, proposal.bond_amount)?;
            
            registry.total_bonds_forfeited += proposal.bond_amount;
        }
        
        // Update registry
        registry.active_proposals = registry.active_proposals.saturating_sub(1);
        
        emit!(ProposalFinalized {
            id: proposal.id,
            status: new_status,
            votes_yes: proposal.votes_yes,
            votes_no: proposal.votes_no,
            votes_abstain: proposal.votes_abstain,
            quorum_reached,
            approved,
            reputation_change,
            bond_returned: return_bond,
            timestamp: current_time,
        });
        
        // ====================================================================
        // EXECUTE IF PASSED
        // ====================================================================
        
        if proposal.status == ProposalStatus::Passed {
            match proposal.proposal_type {
                ProposalType::Sanction => {
                    msg!("Sanction proposal passed - create sanction account separately");
                },
                ProposalType::Treasury => {
                    msg!("Treasury proposal passed - execute transfer separately");
                },
                _ => {}
            }
        }
        
        Ok(())
    }

    // ========================================================================
    // 2.3 COUNTRY SANCTIONS
    // ========================================================================

    // ========================================================================
    // 2.3 COUNTRY SANCTIONS
    // ========================================================================
    // 🟠 SANCTION (>67%) - These functions require 67% supermajority vote

    /// Impose a sanction on a country (called after Sanction proposal passes).
    /// 
    /// 🔴 REQUIRES: SANCTION proposal (>67% approval, 5% quorum, 75K bond)
    /// 
    /// Creates a CountrySanction account that Core program reads.
    /// Core will reduce UBI for citizens of this country.
    /// 
    /// # Important
    /// 
    /// - Sanctions REDUCE UBI, never to zero (minimum 1%)
    /// - Core reads this account but Governance cannot force Core to do anything
    /// - Sanction has expiration date
    pub fn impose_sanction(ctx: Context<ImposeSanction>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let proposal = &ctx.accounts.proposal;
        let sanction = &mut ctx.accounts.country_sanction;
        
        // Verify proposal passed
        require!(
            proposal.status == ProposalStatus::Passed,
            GovernanceError::ProposalNotPassed
        );
        require!(
            proposal.proposal_type == ProposalType::Sanction,
            GovernanceError::WrongProposalType
        );
        
        // Create sanction
        sanction.country_code = proposal.sanction_country;
        sanction.ubi_percentage = proposal.sanction_ubi_pct;
        sanction.imposed_at = current_time;
        sanction.expires_at = current_time + DEFAULT_SANCTION_DURATION;
        sanction.proposal_id = proposal.id;
        sanction.is_active = true;
        sanction.lifted_early = false;
        sanction.lifted_at = 0;
        sanction.bump = ctx.bumps.country_sanction;
        
        emit!(SanctionImposed {
            country_code: sanction.country_code,
            ubi_percentage: sanction.ubi_percentage,
            proposal_id: proposal.id,
            expires_at: sanction.expires_at,
            timestamp: current_time,
        });
        
        msg!(
            "Sanction imposed on country {:?}. UBI reduced to {}%",
            sanction.country_code,
            sanction.ubi_percentage
        );
        
        Ok(())
    }

    /// Lift a sanction early (requires new DAO vote).
    /// 
    /// 🔴 REQUIRES: SANCTION proposal (>67% approval, 5% quorum, 75K bond)
    pub fn lift_sanction(ctx: Context<LiftSanction>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let sanction = &mut ctx.accounts.country_sanction;
        let lift_proposal = &ctx.accounts.lift_proposal;
        
        // Verify lift proposal passed
        require!(
            lift_proposal.status == ProposalStatus::Passed,
            GovernanceError::ProposalNotPassed
        );
        
        // Lift sanction
        sanction.is_active = false;
        sanction.lifted_early = true;
        sanction.lifted_at = current_time;
        
        emit!(SanctionLifted {
            country_code: sanction.country_code,
            lift_proposal_id: lift_proposal.id,
            timestamp: current_time,
        });
        
        msg!("Sanction on country {:?} lifted", sanction.country_code);
        
        Ok(())
    }

    // ========================================================================
    // 2.4 GAS POOL / SPONSORSHIP
    // ========================================================================

    /// Initialize the gas pool.
    pub fn initialize_gas_pool(ctx: Context<InitializeGasPool>) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let gas_pool = &mut ctx.accounts.gas_pool_state;
        
        gas_pool.total_deposited = 0;
        gas_pool.total_subsidized = 0;
        gas_pool.available_balance = 0;
        gas_pool.total_sponsors = 0;
        gas_pool.total_transactions_subsidized = 0;
        gas_pool.emergency_paused = false;
        gas_pool.created_at = current_time;
        gas_pool.bump = ctx.bumps.gas_pool_state;
        
        emit!(GasPoolInitialized {
            timestamp: current_time,
        });
        
        Ok(())
    }

    /// Sponsor the gas pool with SOL.
    /// 
    /// # Sponsor Tiers
    /// 
    /// Based on total contribution:
    /// - Bronze: 1+ SOL - 10% personal allocation, 10% fee discount
    /// - Silver: 10+ SOL - 15% personal, 20% discount
    /// - Gold: 100+ SOL - 20% personal, 40% discount
    /// - Platinum: 1,000+ SOL - 25% personal, 60% discount
    /// - Diamond: 10,000+ SOL - 30% personal, FREE fees
    pub fn sponsor_gas_pool(ctx: Context<SponsorGasPool>, amount: u64) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let gas_pool = &mut ctx.accounts.gas_pool_state;
        let sponsor = &mut ctx.accounts.sponsor_record;
        
        require!(!gas_pool.emergency_paused, GovernanceError::GasPoolPaused);
        require!(amount > 0, GovernanceError::InvalidAmount);
        
        // Calculate tier
        let total_contribution = sponsor.total_contributed + amount;
        let new_tier = calculate_sponsor_tier(total_contribution);
        let personal_pct = get_personal_allocation_pct(&new_tier);
        let personal_amount = (amount * personal_pct) / 100;
        let pool_amount = amount - personal_amount;
        
        // Transfer SOL
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.sponsor.key(),
            &ctx.accounts.gas_pool_vault.key(),
            amount,
        );
        
        anchor_lang::solana_program::program::invoke(
            &transfer_ix,
            &[
                ctx.accounts.sponsor.to_account_info(),
                ctx.accounts.gas_pool_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        )?;
        
        // Update sponsor record
        let is_new = sponsor.total_contributed == 0;
        let old_tier = sponsor.tier.clone();
        
        sponsor.sponsor = ctx.accounts.sponsor.key();
        sponsor.tier = new_tier.clone();
        sponsor.total_contributed = total_contribution;
        sponsor.personal_allocation += personal_amount;
        sponsor.pool_contribution += pool_amount;
        
        if is_new {
            sponsor.sponsored_at = current_time;
            sponsor.bump = ctx.bumps.sponsor_record;
            gas_pool.total_sponsors += 1;
        }
        
        // Update gas pool
        gas_pool.total_deposited += amount;
        gas_pool.available_balance += pool_amount;
        
        emit!(SponsorshipReceived {
            sponsor: ctx.accounts.sponsor.key(),
            amount,
            personal_allocation: personal_amount,
            pool_contribution: pool_amount,
            new_tier: new_tier.clone(),
            timestamp: current_time,
        });
        
        if new_tier != old_tier && !is_new {
            emit!(SponsorTierUpgraded {
                sponsor: ctx.accounts.sponsor.key(),
                from_tier: old_tier,
                to_tier: new_tier,
                total_contributed: total_contribution,
                timestamp: current_time,
            });
        }
        
        Ok(())
    }

    // ========================================================================
    // 2.5 FEE MANAGEMENT
    // ========================================================================
    // 🔴 CONSTITUTIONAL (>75%) - Fee changes require high supermajority

    /// Update fee parameters (requires Constitutional proposal).
    /// 
    /// 🟠 REQUIRES: CONSTITUTIONAL proposal (>75% approval, 10% quorum, 100K bond)
    pub fn update_fee_parameters(
        ctx: Context<UpdateFeeParameters>,
        new_base_rate: u64,
        new_burn_pct: u64,
    ) -> Result<()> {
        let current_time = Clock::get()?.unix_timestamp;
        let fee_state = &mut ctx.accounts.fee_state;
        let proposal = &ctx.accounts.proposal;
        
        // Verify proposal passed
        require!(
            proposal.status == ProposalStatus::Passed,
            GovernanceError::ProposalNotPassed
        );
        require!(
            proposal.proposal_type == ProposalType::Constitutional,
            GovernanceError::WrongProposalType
        );
        
        // Validate parameters
        require!(new_base_rate <= 100, GovernanceError::FeeTooHigh); // Max 1%
        require!(new_burn_pct <= 100, GovernanceError::InvalidBurnPercentage);
        
        // Update
        fee_state.base_fee_rate = new_base_rate;
        fee_state.burn_percentage = new_burn_pct;
        fee_state.last_updated = current_time;
        
        emit!(FeeParametersUpdated {
            base_fee_rate: new_base_rate,
            burn_percentage: new_burn_pct,
            proposal_id: proposal.id,
            timestamp: current_time,
        });
        
        Ok(())
    }
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    SECTION 3: ACCOUNT STRUCTURES                          ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

// ============================================================================
// 3.1 GOVERNANCE STATE
// ============================================================================

#[account]
pub struct GovernanceState {
    pub authority: Pubkey,
    pub core_program: Pubkey,
    pub is_initialized: bool,
    pub launch_timestamp: i64,
    pub bump: u8,
}

impl GovernanceState {
    pub const SIZE: usize = 8 + 32 + 32 + 1 + 8 + 1;
}

// ============================================================================
// 3.2 FEE STATE (read by Core program)
// ============================================================================

/// Fee parameters that Core program reads.
/// 
/// This account is created by Governance but read by Core.
/// Core uses these values to calculate transfer fees.
#[account]
pub struct FeeState {
    /// Base fee rate in basis points
    pub base_fee_rate: u64,
    
    /// Percentage of fees that are burned
    pub burn_percentage: u64,
    
    /// Fee multiplier for active users (100 = 1x)
    pub active_multiplier: u64,
    
    /// Fee multiplier for normal users
    pub normal_multiplier: u64,
    
    /// Fee multiplier for inactive users
    pub inactive_multiplier: u64,
    
    /// Fee multiplier for dormant users
    pub dormant_multiplier: u64,
    
    /// Last update timestamp
    pub last_updated: i64,
    
    /// PDA bump
    pub bump: u8,
}

impl FeeState {
    pub const SIZE: usize = 8 + 8 + 8 + 8 + 8 + 8 + 8 + 8 + 1;
}

// ============================================================================
// 3.3 PROPOSAL
// ============================================================================

#[account]
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub title: [u8; 64],
    pub description_hash: [u8; 32],
    pub bond_amount: u64,
    pub bond_resolved: bool,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub votes_abstain: u64,
    pub total_voters: u64,
    pub quorum_required: u64,
    pub approval_threshold: u64,
    pub created_at: i64,
    pub voting_ends_at: i64,
    pub executed_at: i64,
    pub treasury_amount: u64,
    pub treasury_recipient: Pubkey,
    pub sanction_country: [u8; 3],
    pub sanction_ubi_pct: u8,
    pub bump: u8,
}

impl Proposal {
    pub const SIZE: usize = 8 +  // discriminator
        8 +     // id
        32 +    // proposer
        1 +     // proposal_type
        1 +     // status
        64 +    // title
        32 +    // description_hash
        8 +     // bond_amount
        1 +     // bond_resolved
        8 +     // votes_yes
        8 +     // votes_no
        8 +     // votes_abstain
        8 +     // total_voters
        8 +     // quorum_required
        8 +     // approval_threshold
        8 +     // created_at
        8 +     // voting_ends_at
        8 +     // executed_at
        8 +     // treasury_amount
        32 +    // treasury_recipient
        3 +     // sanction_country
        1 +     // sanction_ubi_pct
        1;      // bump
}

// ============================================================================
// 3.4 VOTE RECORD
// ============================================================================

#[account]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal_id: u64,
    pub choice: u8,
    pub voted_at: i64,
    pub has_voted: bool,
    pub bump: u8,
}

impl VoteRecord {
    pub const SIZE: usize = 8 + 32 + 8 + 1 + 8 + 1 + 1;
}

// ============================================================================
// 3.5 PROPOSAL REGISTRY
// ============================================================================

#[account]
pub struct ProposalRegistry {
    pub next_proposal_id: u64,
    pub total_proposals: u64,
    pub active_proposals: u32,
    pub total_bonds_forfeited: u64,
    pub total_bonds_returned: u64,
    pub bump: u8,
}

impl ProposalRegistry {
    pub const SIZE: usize = 8 + 8 + 8 + 4 + 8 + 8 + 1;
}

// ============================================================================
// 3.6 COUNTRY SANCTION (read by Core program)
// ============================================================================

/// Sanction record that Core program reads.
/// 
/// When Core processes a claim, it checks if a sanction exists
/// for the user's citizenship. If so, UBI is reduced.
#[account]
pub struct CountrySanction {
    /// ISO 3166-1 alpha-3 country code
    pub country_code: [u8; 3],
    
    /// UBI percentage (1-99, never 0 or 100)
    pub ubi_percentage: u8,
    
    /// When sanction was imposed
    pub imposed_at: i64,
    
    /// When sanction expires
    pub expires_at: i64,
    
    /// Proposal that created this sanction
    pub proposal_id: u64,
    
    /// Whether sanction is currently active
    pub is_active: bool,
    
    /// Whether sanction was lifted early
    pub lifted_early: bool,
    
    /// When sanction was lifted (if applicable)
    pub lifted_at: i64,
    
    /// PDA bump
    pub bump: u8,
}

impl CountrySanction {
    pub const SIZE: usize = 8 + 3 + 1 + 8 + 8 + 8 + 1 + 1 + 8 + 1;
}

// ============================================================================
// 3.7 PROPOSER STATE (extension of Core's UserState)
// ============================================================================

/// Governance-specific user data.
/// 
/// This extends Core's UserState with governance-related fields.
#[account]
pub struct ProposerState {
    pub user: Pubkey,
    pub proposal_reputation: i32,
    pub proposals_created: u32,
    pub proposals_passed: u32,
    pub proposals_rejected: u32,
    pub proposals_expired: u32,
    pub total_votes_cast: u64,
    pub bump: u8,
}

impl ProposerState {
    pub const SIZE: usize = 8 + 32 + 4 + 4 + 4 + 4 + 4 + 8 + 1;
}

// ============================================================================
// 3.8 GAS POOL STATE
// ============================================================================

#[account]
pub struct GasPoolState {
    pub total_deposited: u64,
    pub total_subsidized: u64,
    pub available_balance: u64,
    pub total_sponsors: u64,
    pub total_transactions_subsidized: u64,
    pub emergency_paused: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl GasPoolState {
    pub const SIZE: usize = 8 + 8 + 8 + 8 + 8 + 8 + 1 + 8 + 1;
}

// ============================================================================
// 3.9 SPONSOR RECORD
// ============================================================================

#[account]
pub struct SponsorRecord {
    pub sponsor: Pubkey,
    pub tier: SponsorTier,
    pub total_contributed: u64,
    pub personal_allocation: u64,
    pub pool_contribution: u64,
    pub sponsored_at: i64,
    pub bump: u8,
}

impl SponsorRecord {
    pub const SIZE: usize = 8 + 32 + 1 + 8 + 8 + 8 + 8 + 1;
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    SECTION 4: ENUMS                                       ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalType {
    Standard,
    Treasury,
    Constitutional,
    Sanction,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Expired,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum SponsorTier {
    None,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    SECTION 5: CONTEXT STRUCTURES                          ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

#[derive(Accounts)]
pub struct InitializeGovernance<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = GovernanceState::SIZE,
        seeds = [b"governance"],
        bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        init,
        payer = authority,
        space = FeeState::SIZE,
        seeds = [b"fee_state"],
        bump
    )]
    pub fee_state: Account<'info, FeeState>,
    
    #[account(
        init,
        payer = authority,
        space = ProposalRegistry::SIZE,
        seeds = [b"proposal_registry"],
        bump
    )]
    pub proposal_registry: Account<'info, ProposalRegistry>,
    
    /// CHECK: Core program address
    pub core_program: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"proposer", proposer.key().as_ref()],
        bump = proposer_state.bump
    )]
    pub proposer_state: Account<'info, ProposerState>,
    
    #[account(
        init,
        payer = proposer,
        space = Proposal::SIZE,
        seeds = [b"proposal", proposal_registry.next_proposal_id.to_le_bytes().as_ref()],
        bump
    )]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        mut,
        seeds = [b"proposal_registry"],
        bump = proposal_registry.bump
    )]
    pub proposal_registry: Account<'info, ProposalRegistry>,
    
    /// Core protocol state - needed to read total_users for dynamic quorum
    /// CHECK: Verified by seeds from Core program
    #[account(
        seeds = [b"protocol_state"],
        bump,
        seeds::program = governance_state.core_program
    )]
    pub core_protocol_state: AccountInfo<'info>,
    
    #[account(
        seeds = [b"governance_state"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(mut)]
    pub proposer_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    #[account(mut)]
    pub bond_escrow: Account<'info, anchor_spl::token::TokenAccount>,
    
    pub token_program: Program<'info, anchor_spl::token::Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct VoteOnProposal<'info> {
    pub voter: Signer<'info>,
    
    /// CHECK: Voter's Core UserState
    pub voter_state: AccountInfo<'info>,
    
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        init,
        payer = voter,
        space = VoteRecord::SIZE,
        seeds = [b"vote", proposal.key().as_ref(), voter.key().as_ref()],
        bump
    )]
    pub vote_record: Account<'info, VoteRecord>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
    
    #[account(mut)]
    pub proposer_state: Account<'info, ProposerState>,
    
    #[account(
        seeds = [b"governance"],
        bump = governance_state.bump
    )]
    pub governance_state: Account<'info, GovernanceState>,
    
    #[account(
        mut,
        seeds = [b"proposal_registry"],
        bump = proposal_registry.bump
    )]
    pub proposal_registry: Account<'info, ProposalRegistry>,
    
    #[account(mut)]
    pub bond_escrow: Account<'info, anchor_spl::token::TokenAccount>,
    
    #[account(mut)]
    pub proposer_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    #[account(mut)]
    pub treasury_token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

#[derive(Accounts)]
pub struct ImposeSanction<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub proposal: Account<'info, Proposal>,
    
    #[account(
        init,
        payer = authority,
        space = CountrySanction::SIZE,
        seeds = [b"sanction", proposal.sanction_country.as_ref()],
        bump
    )]
    pub country_sanction: Account<'info, CountrySanction>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct LiftSanction<'info> {
    #[account(mut)]
    pub country_sanction: Account<'info, CountrySanction>,
    
    pub lift_proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct InitializeGasPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = GasPoolState::SIZE,
        seeds = [b"gas_pool"],
        bump
    )]
    pub gas_pool_state: Account<'info, GasPoolState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SponsorGasPool<'info> {
    #[account(mut)]
    pub sponsor: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"gas_pool"],
        bump = gas_pool_state.bump
    )]
    pub gas_pool_state: Account<'info, GasPoolState>,
    
    #[account(
        init_if_needed,
        payer = sponsor,
        space = SponsorRecord::SIZE,
        seeds = [b"sponsor", sponsor.key().as_ref()],
        bump
    )]
    pub sponsor_record: Account<'info, SponsorRecord>,
    
    /// CHECK: Gas pool vault PDA
    #[account(mut)]
    pub gas_pool_vault: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateFeeParameters<'info> {
    #[account(
        mut,
        seeds = [b"fee_state"],
        bump = fee_state.bump
    )]
    pub fee_state: Account<'info, FeeState>,
    
    pub proposal: Account<'info, Proposal>,
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    SECTION 6: HELPER FUNCTIONS                            ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

fn calculate_sponsor_tier(total_contribution: u64) -> SponsorTier {
    if total_contribution >= DIAMOND_THRESHOLD {
        SponsorTier::Diamond
    } else if total_contribution >= PLATINUM_THRESHOLD {
        SponsorTier::Platinum
    } else if total_contribution >= GOLD_THRESHOLD {
        SponsorTier::Gold
    } else if total_contribution >= SILVER_THRESHOLD {
        SponsorTier::Silver
    } else if total_contribution >= BRONZE_THRESHOLD {
        SponsorTier::Bronze
    } else {
        SponsorTier::None
    }
}

fn get_personal_allocation_pct(tier: &SponsorTier) -> u64 {
    match tier {
        SponsorTier::None => 0,
        SponsorTier::Bronze => BRONZE_PERSONAL_PCT,
        SponsorTier::Silver => SILVER_PERSONAL_PCT,
        SponsorTier::Gold => GOLD_PERSONAL_PCT,
        SponsorTier::Platinum => PLATINUM_PERSONAL_PCT,
        SponsorTier::Diamond => DIAMOND_PERSONAL_PCT,
    }
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    SECTION 7: EVENTS                                      ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

#[event]
pub struct GovernanceInitialized {
    pub authority: Pubkey,
    pub core_program: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ProposalCreated {
    pub id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub title: [u8; 64],
    pub bond_amount: u64,
    pub quorum_required: u64,
    pub voting_ends_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct VoteCast {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub choice: u8,
    pub timestamp: i64,
}

#[event]
pub struct ProposalFinalized {
    pub id: u64,
    pub status: ProposalStatus,
    pub votes_yes: u64,
    pub votes_no: u64,
    pub votes_abstain: u64,
    pub quorum_reached: bool,
    pub approved: bool,
    pub reputation_change: i32,
    pub bond_returned: bool,
    pub timestamp: i64,
}

#[event]
pub struct SanctionImposed {
    pub country_code: [u8; 3],
    pub ubi_percentage: u8,
    pub proposal_id: u64,
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct SanctionLifted {
    pub country_code: [u8; 3],
    pub lift_proposal_id: u64,
    pub timestamp: i64,
}

#[event]
pub struct GasPoolInitialized {
    pub timestamp: i64,
}

#[event]
pub struct SponsorshipReceived {
    pub sponsor: Pubkey,
    pub amount: u64,
    pub personal_allocation: u64,
    pub pool_contribution: u64,
    pub new_tier: SponsorTier,
    pub timestamp: i64,
}

#[event]
pub struct SponsorTierUpgraded {
    pub sponsor: Pubkey,
    pub from_tier: SponsorTier,
    pub to_tier: SponsorTier,
    pub total_contributed: u64,
    pub timestamp: i64,
}

#[event]
pub struct FeeParametersUpdated {
    pub base_fee_rate: u64,
    pub burn_percentage: u64,
    pub proposal_id: u64,
    pub timestamp: i64,
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    SECTION 8: ERROR CODES                                 ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

#[error_code]
pub enum GovernanceError {
    #[msg("Proposer is banned due to low reputation")]
    ProposerBanned,
    
    #[msg("Invalid sanction percentage (must be 1-99)")]
    InvalidSanctionPercentage,
    
    #[msg("Voter is not verified")]
    VoterNotVerified,
    
    #[msg("Proposal is not active")]
    ProposalNotActive,
    
    #[msg("Voting period has ended")]
    VotingEnded,
    
    #[msg("Invalid vote choice (must be 0, 1, or 2)")]
    InvalidVoteChoice,
    
    #[msg("Already voted on this proposal")]
    AlreadyVoted,
    
    #[msg("Voting period has not ended yet")]
    VotingNotEnded,
    
    #[msg("Proposal has not passed")]
    ProposalNotPassed,
    
    #[msg("Wrong proposal type for this action")]
    WrongProposalType,
    
    #[msg("Gas pool is paused")]
    GasPoolPaused,
    
    #[msg("Invalid amount")]
    InvalidAmount,
    
    #[msg("Fee rate too high (max 1%)")]
    FeeTooHigh,
    
    #[msg("Invalid burn percentage (must be 0-100)")]
    InvalidBurnPercentage,
}

// ╔═══════════════════════════════════════════════════════════════════════════╗
// ║                                                                           ║
// ║                    END OF AGORA GOVERNANCE                                ║
// ║                                                                           ║
// ║   This program is UPGRADEABLE by DAO vote.                                ║
// ║   Constitutional proposals (67% supermajority) can modify this code.      ║
// ║                                                                           ║
// ║   But remember: Core's 100 AGORA/day can NEVER be changed.                ║
// ║   Governance can only influence AROUND that sacred constant.              ║
// ║                                                                           ║
// ╚═══════════════════════════════════════════════════════════════════════════╝

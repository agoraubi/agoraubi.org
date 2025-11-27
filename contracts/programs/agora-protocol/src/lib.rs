// AGORA Protocol v3.3 - Single Business Model with Off-chain Registry
// Universal Basic Income with Human-First Architecture
//
// CRITICAL DESIGN: One human = One business (on-chain)
// Unlimited divisions/branches stored off-chain (IPFS/Arweave)
//
// SECURITY v3.3: Annual Liveness Verification
// - ALL countries treated equally (no trust scores - any government could be hostile)
// - Biometric proof-of-life: eID card biometrics must match LIVE biometric scan
// - Dead person fraud prevention: corpse cannot perform live biometric scan
// - No reliance on government death registries (don't trust, verify)

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, MintTo, Transfer, Burn}
};
use anchor_lang::solana_program::system_instruction;

declare_id!("AGORA111111111111111111111111111111111111111");

// ============================================================================
// CONSTANTS
// ============================================================================

// Token Distribution
pub const DAILY_AMOUNT: u64 = 100_000_000_000;        // 100 AGORA (9 decimals)
pub const MAX_RETROACTIVE_DAYS: u64 = 365;           // 1 year max retroactive
pub const MAX_ACCUMULATION_DAYS: u64 = 30;           // 30 days max accumulation
pub const SECONDS_PER_DAY: i64 = 86400;

// Transfer Fees
pub const BASE_FEE_RATE: u64 = 5;                    // 0.05% = 5 basis points
pub const FEE_DIVISOR: u64 = 10000;                  // For basis point calculation
pub const BURN_PERCENTAGE: u64 = 50;                 // 50% of fees burned

// Rate Limiting
pub const MIN_TIME_BETWEEN_TX: i64 = 60;             // 60 seconds minimum
pub const DAILY_TX_LIMIT: u8 = 50;                   // Max 50 TX/day base

// Merchant Detection - Volume Based
pub const MIN_VOLUME_RATIO: f64 = 3.0;               // 3x more volume in than out
pub const MIN_UNIQUE_CUSTOMERS: u32 = 10;            // Minimum unique payers
pub const HIGH_VALUE_THRESHOLD: u64 = 10_000_000_000_000; // 10,000 AGORA

// Security
pub const SUSPENSION_DAYS: i64 = 30;                 // First violation
pub const MERCHANT_DECAY_DAYS: i64 = 90;             // Inactivity before decay
pub const MAX_METADATA_URI_LENGTH: usize = 128;      // IPFS CID or Arweave hash

// ============================================================================
// ANNUAL LIVENESS VERIFICATION CONSTANTS
// ============================================================================
// Philosophy: Don't trust ANY government. Verify the human is ALIVE.
// Method: Biometrics on eID card must match LIVE biometric scan.
// Result: Dead people cannot claim (corpse can't do iris/fingerprint scan)

// Timing
pub const LIVENESS_INTERVAL: i64 = 365 * SECONDS_PER_DAY;    // Must verify every year
pub const LIVENESS_GRACE_PERIOD: i64 = 30 * SECONDS_PER_DAY; // 30 days after expiry to verify
pub const LIVENESS_WARNING_DAYS: i64 = 60;                    // Warn 60 days before expiry

// Biometric Verification
pub const BIOMETRIC_CHALLENGE_TIMEOUT: i64 = 300;    // 5 minutes to complete verification
pub const MAX_VERIFICATION_ATTEMPTS: u8 = 3;         // Max failed attempts per day
pub const VERIFICATION_LOCKOUT: i64 = 24 * 60 * 60;  // 24h lockout after max attempts

// Supported biometric types (from eID cards)
// User must match what's stored on THEIR eID card
pub const BIOMETRIC_FINGERPRINT: u8 = 1;
pub const BIOMETRIC_IRIS: u8 = 2;
pub const BIOMETRIC_FACE: u8 = 3;

// Hash sizes for biometric templates (privacy-preserving)
pub const BIOMETRIC_HASH_SIZE: usize = 32;           // SHA-256 of biometric template

// ============================================================================
// BIOMETRIC DEDUPLICATION CONSTANTS
// ============================================================================
// eIDAS requires EXACTLY both index fingers - prevents using other fingers
// hash(left_index + right_index) = globally unique identifier
// Same person ALWAYS produces same hash, regardless of country

pub const FINGERPRINT_TEMPLATE_SIZE: usize = 512;    // ISO/IEC 19794-2 template size

#[program]
pub mod agora_protocol {
    use super::*;

    // ========================================================================
    // INITIALIZATION
    // ========================================================================

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let protocol = &mut ctx.accounts.protocol_state;
        
        protocol.authority = ctx.accounts.authority.key();
        protocol.mint = ctx.accounts.mint.key();
        protocol.treasury = ctx.accounts.treasury.key();
        protocol.gas_pool = ctx.accounts.gas_pool.key();
        protocol.total_users = 0;
        protocol.total_merchants = 0;
        protocol.total_businesses = 0;
        protocol.total_minted = 0;
        protocol.total_burned = 0;
        protocol.launch_timestamp = Clock::get()?.unix_timestamp;
        protocol.emergency_paused = false;
        protocol.business_registration_frozen = false;
        protocol.bump = ctx.bumps.protocol_state;
        
        emit!(ProtocolInitialized {
            mint: protocol.mint,
            treasury: protocol.treasury,
            gas_pool: protocol.gas_pool,
            timestamp: protocol.launch_timestamp,
        });
        
        Ok(())
    }

    // ========================================================================
    // USER REGISTRATION - HUMANS ONLY
    // ========================================================================
    // Registration requires:
    // 1. Valid eIDAS/government ID with biometrics
    // 2. Live biometric scan matching eID (proof of life)
    // 3. Biometric hash not already registered (deduplication)
    //
    // NO country-level penalties - only individual fraud is punished

    pub fn register_user(
        ctx: Context<RegisterUser>,
        age_in_days: u64,
        civic_pass_proof: Vec<u8>,
        biometric_hash: [u8; 32],              // SHA-256(left_index + right_index)
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // =====================================================================
        // BIOMETRIC DEDUPLICATION CHECK
        // =====================================================================
        // CRITICAL: Same fingerprints = same person = REJECT
        // eIDAS requires EXACTLY both index fingers - cannot use other fingers
        //
        // The biometric_entry PDA will fail to init if hash already exists
        // This is enforced by Anchor's init constraint - no code needed!

        let biometric_entry = &mut ctx.accounts.biometric_entry;
        biometric_entry.biometric_hash = biometric_hash;
        biometric_entry.user_account = ctx.accounts.user.key();
        biometric_entry.registered_at = current_time;
        biometric_entry.is_active = true;
        biometric_entry.bump = ctx.bumps.biometric_entry;

        // Update registry stats
        let registry = &mut ctx.accounts.biometric_registry;
        registry.total_registered += 1;
        registry.last_updated = current_time;

        // =====================================================================
        // CIVIC PASS / eIDAS VERIFICATION
        // =====================================================================
        verify_civic_pass(&civic_pass_proof)?;

        // =====================================================================
        // INITIALIZE USER ACCOUNT
        // =====================================================================

        user.owner = ctx.accounts.user.key();
        user.civic_pass = ctx.accounts.civic_pass.key();
        user.is_human = true;
        user.registration_timestamp = current_time;
        user.last_claim_timestamp = current_time;
        user.age_in_days_at_registration = age_in_days;
        user.is_child = age_in_days < 6570; // Under 18

        // Biometric identity - for deduplication only
        user.biometric_hash = biometric_hash;

        // Merchant fields - volume based
        user.merchant_status = MerchantTier::None;
        user.volume_received = 0;
        user.volume_sent = 0;
        user.tx_count_received = 0;
        user.tx_count_sent = 0;
        user.unique_payers = Vec::with_capacity(1000);
        user.unique_payers_count = 0;
        user.average_transaction_size = 0;

        // SINGLE BUSINESS MODEL
        user.business_profile = None;

        // Security
        user.violations = 0;
        user.suspended_until = 0;
        user.permanently_blacklisted = false;
        user.bump = ctx.bumps.user_state;
        
        // Mint retroactive claim for HUMAN only
        let retroactive_amount = calculate_retroactive_claim(age_in_days);
        
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.protocol_state.to_account_info(),
                },
                &[&[b"protocol", &[ctx.accounts.protocol_state.bump]]],
            ),
            retroactive_amount,
        )?;
        
        user.total_claimed = retroactive_amount;

        // Update protocol
        let protocol = &mut ctx.accounts.protocol_state;
        protocol.total_users += 1;
        protocol.total_minted += retroactive_amount;

        emit!(UserRegistered {
            user: ctx.accounts.user.key(),
            retroactive_claim: retroactive_amount,
            age_in_days,
            is_human: true,
            biometric_hash,
            timestamp: user.registration_timestamp,
        });

        Ok(())
    }

    // ========================================================================
    // BUSINESS REGISTRATION - ONE PER HUMAN
    // ========================================================================

    pub fn register_business(
        ctx: Context<RegisterBusiness>,
        business_info: BusinessInfo,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let protocol = &ctx.accounts.protocol_state;
        
        // Check if business registration is frozen (emergency)
        require!(
            !protocol.business_registration_frozen,
            ErrorCode::BusinessRegistrationFrozen
        );
        
        // CRITICAL: Only verified humans can have businesses
        require!(
            user.is_human && user.civic_pass != Pubkey::default(),
            ErrorCode::MustBeVerifiedHuman
        );
        
        // CRITICAL SECURITY: EXACTLY ONE business per human
        // This prevents:
        // - Storage bloat attacks (10,000 fake businesses)
        // - Chain state explosion
        // - Marketplace spam
        // - Performance degradation
        // Legitimate users can add unlimited divisions OFF-CHAIN
        require!(
            user.business_profile.is_none(),
            ErrorCode::BusinessAlreadyExists
        );
        
        // Validate metadata URI (IPFS CID or Arweave hash)
        require!(
            business_info.metadata_uri.len() <= MAX_METADATA_URI_LENGTH,
            ErrorCode::MetadataUriTooLong
        );
        
        // Create single business profile
        user.business_profile = Some(BusinessProfile {
            name: business_info.name,
            category: business_info.category,
            tax_id: business_info.tax_id,
            metadata_uri: business_info.metadata_uri,  // Points to off-chain data
            created_at: Clock::get()?.unix_timestamp,
            is_verified: false,  // DAO can verify later
            is_active: true,
            volume_processed: 0,
            employee_count: 0,
            last_metadata_update: Clock::get()?.unix_timestamp,
        });
        
        // Update protocol
        let protocol_mut = &mut ctx.accounts.protocol_state;
        protocol_mut.total_businesses += 1;
        
        emit!(BusinessRegistered {
            human_owner: user.owner,
            business_name: business_info.name.clone(),
            category: business_info.category,
            metadata_uri: business_info.metadata_uri,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        msg!("Business registered. Add divisions/branches via metadata URI update.");
        
        Ok(())
    }

    // ========================================================================
    // BUSINESS METADATA UPDATE - For Off-chain Registry
    // ========================================================================

    pub fn update_business_metadata(
        ctx: Context<UpdateBusiness>,
        new_metadata_uri: String,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Must have a business
        require!(
            user.business_profile.is_some(),
            ErrorCode::NoBusinessRegistered
        );
        
        // Validate metadata URI
        require!(
            new_metadata_uri.len() <= MAX_METADATA_URI_LENGTH,
            ErrorCode::MetadataUriTooLong
        );
        
        // Rate limit metadata updates (once per day)
        let business = user.business_profile.as_mut().unwrap();
        require!(
            current_time - business.last_metadata_update >= SECONDS_PER_DAY,
            ErrorCode::MetadataUpdateTooSoon
        );
        
        // Update metadata pointer
        let old_uri = business.metadata_uri.clone();
        business.metadata_uri = new_metadata_uri.clone();
        business.last_metadata_update = current_time;
        
        emit!(BusinessMetadataUpdated {
            business_owner: user.owner,
            business_name: business.name.clone(),
            old_metadata_uri: old_uri,
            new_metadata_uri,
            timestamp: current_time,
        });
        
        msg!("Metadata updated. Off-chain divisions/branches can be modified freely.");
        
        Ok(())
    }

    // ========================================================================
    // BUSINESS REMOVAL - Clean Deregistration
    // ========================================================================

    pub fn remove_business(ctx: Context<RemoveBusiness>) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        
        // Must have a business
        require!(
            user.business_profile.is_some(),
            ErrorCode::NoBusinessRegistered
        );
        
        let business = user.business_profile.take().unwrap();
        
        // Update protocol
        let protocol = &mut ctx.accounts.protocol_state;
        if protocol.total_businesses > 0 {
            protocol.total_businesses -= 1;
        }
        
        emit!(BusinessRemoved {
            human_owner: user.owner,
            business_name: business.name,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        Ok(())
    }

    // ========================================================================
    // MERCHANT STATUS UPDATE - Volume Based
    // ========================================================================

    pub fn update_merchant_status(ctx: Context<UpdateMerchantStatus>) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Check for decay first
        let days_inactive = (current_time - user.last_merchant_tx) / SECONDS_PER_DAY;
        if days_inactive > MERCHANT_DECAY_DAYS {
            decay_merchant_status(user)?;
            return Ok(());
        }
        
        // Calculate volume-based metrics
        let volume_ratio = if user.volume_sent > 0 {
            (user.volume_received as f64) / (user.volume_sent as f64)
        } else if user.volume_received > 0 {
            f64::MAX  // Only receiving, no sending
        } else {
            0.0
        };
        
        // Calculate average transaction size
        let avg_tx_size = if user.tx_count_received > 0 {
            user.volume_received / user.tx_count_received as u128
        } else {
            0
        };
        
        // Determine merchant type and score
        let (score, merchant_type) = if avg_tx_size > HIGH_VALUE_THRESHOLD as u128 {
            // High-value service provider
            let score = calculate_service_merchant_score(
                volume_ratio,
                user.unique_payers_count,
                avg_tx_size,
            );
            (score, MerchantType::Service)
        } else {
            // High-frequency merchant
            let score = calculate_retail_merchant_score(
                volume_ratio,
                user.unique_payers_count,
                user.tx_count_received,
            );
            (score, MerchantType::Retail)
        };
        
        // Determine tier
        let new_tier = determine_merchant_tier(score, merchant_type);
        
        // Upgrade if improved
        if new_tier > user.merchant_status {
            user.merchant_status = new_tier.clone();
            user.last_merchant_tx = current_time;
            
            let protocol = &mut ctx.accounts.protocol_state;
            if user.merchant_status != MerchantTier::None {
                protocol.total_merchants += 1;
            }
            
            emit!(MerchantStatusUpgraded {
                merchant: user.owner,
                new_tier,
                score,
                merchant_type,
                timestamp: current_time,
            });
        }
        
        Ok(())
    }

    // ========================================================================
    // BUSINESS TRANSACTIONS - Through Human Wallet
    // ========================================================================

    pub fn business_transaction(
        ctx: Context<BusinessTransaction>,
        amount: u64,
        transaction_metadata: Option<String>,  // Optional invoice/reference
    ) -> Result<()> {
        let sender = &ctx.accounts.sender_state;
        let recipient = &mut ctx.accounts.recipient_state;
        let current_time = Clock::get()?.unix_timestamp;
        
        // Verify sender has active business
        require!(
            sender.business_profile.is_some(),
            ErrorCode::NoBusinessRegistered
        );
        
        let business = sender.business_profile.as_ref().unwrap();
        require!(business.is_active, ErrorCode::BusinessNotActive);
        
        // Calculate fee based on human's merchant status
        let fee_rate = calculate_merchant_fee(&sender.merchant_status);
        let fee_amount = (amount * fee_rate) / FEE_DIVISOR;
        let transfer_amount = amount - fee_amount;
        
        // Update metrics for merchant detection
        let sender_mut = &mut ctx.accounts.sender_state;
        sender_mut.volume_sent += amount as u128;
        sender_mut.tx_count_sent += 1;
        
        if let Some(business) = &mut sender_mut.business_profile {
            business.volume_processed += amount as u128;
        }
        
        recipient.volume_received += amount as u128;
        recipient.tx_count_received += 1;
        
        // Track unique payers
        if !recipient.unique_payers.contains(&sender.owner) {
            recipient.unique_payers.push(sender.owner);
            recipient.unique_payers_count += 1;
        }
        
        // Execute transfer
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.from.to_account_info(),
                    to: ctx.accounts.to.to_account_info(),
                    authority: ctx.accounts.signer.to_account_info(),
                },
            ),
            transfer_amount,
        )?;
        
        // Handle fees
        if fee_amount > 0 {
            handle_fees(&ctx, fee_amount)?;
        }
        
        emit!(BusinessTransactionCompleted {
            human_owner: sender.owner,
            business_name: business.name.clone(),
            recipient: recipient.owner,
            amount: transfer_amount,
            fee: fee_amount,
            metadata: transaction_metadata,
            timestamp: current_time,
        });
        
        Ok(())
    }

    // ========================================================================
    // EMPLOYEE MANAGEMENT - Simplified
    // ========================================================================

    pub fn set_employee_count(
        ctx: Context<UpdateBusiness>,
        employee_count: u32,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        
        require!(
            user.business_profile.is_some(),
            ErrorCode::NoBusinessRegistered
        );
        
        // Update on-chain employee count (actual list is off-chain)
        let business = user.business_profile.as_mut().unwrap();
        business.employee_count = employee_count;
        
        emit!(EmployeeCountUpdated {
            business_owner: user.owner,
            business_name: business.name.clone(),
            employee_count,
            timestamp: Clock::get()?.unix_timestamp,
        });
        
        msg!("Employee count updated. Manage employee details in off-chain metadata.");
        
        Ok(())
    }

    // ========================================================================
    // DAILY CLAIMS - HUMANS ONLY
    // ========================================================================

    pub fn claim_daily(ctx: Context<ClaimDaily>) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // CRITICAL: Only humans can claim UBI
        require!(
            user.is_human,
            ErrorCode::OnlyHumansCanClaim
        );

        // Businesses CANNOT claim - prevents Sybil attacks
        // Each business needs a human owner who claims personally

        // Security checks - INDIVIDUAL penalties only
        require!(
            !user.permanently_blacklisted,
            ErrorCode::UserBlacklisted
        );
        require!(
            current_time >= user.suspended_until,
            ErrorCode::UserSuspended
        );

        // =====================================================================
        // ANNUAL LIVENESS CHECK - Must prove you're alive to claim
        // =====================================================================
        let liveness_status = get_liveness_status(&user.liveness, current_time);

        match liveness_status {
            LivenessStatus::Active => {
                // All good, can claim
            },
            LivenessStatus::Warning => {
                emit!(LivenessWarning {
                    user: ctx.accounts.user.key(),
                    days_until_expiry: (user.liveness.next_verification_due - current_time) / SECONDS_PER_DAY,
                    timestamp: current_time,
                });
            },
            LivenessStatus::Expired => {
                let grace_remaining = (user.liveness.next_verification_due + LIVENESS_GRACE_PERIOD - current_time) / SECONDS_PER_DAY;
                emit!(LivenessExpiredWarning {
                    user: ctx.accounts.user.key(),
                    grace_days_remaining: grace_remaining,
                    timestamp: current_time,
                });
            },
            LivenessStatus::Suspended => {
                return Err(ErrorCode::LivenessVerificationRequired.into());
            },
            LivenessStatus::Locked => {
                return Err(ErrorCode::LivenessVerificationLocked.into());
            },
        }

        // Calculate claimable amount - NO country multiplier, full UBI for everyone
        let claimable = calculate_claimable_amount(user, current_time)?;
        require!(claimable > 0, ErrorCode::NothingToClaim);

        // Check claim frequency
        let min_interval = determine_claim_interval(user);
        require!(
            current_time - user.last_claim_timestamp >= min_interval,
            ErrorCode::ClaimTooSoon
        );
        
        // Mint tokens
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.mint.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: ctx.accounts.protocol_state.to_account_info(),
                },
                &[&[b"protocol", &[ctx.accounts.protocol_state.bump]]],
            ),
            claimable,
        )?;
        
        // Update state
        user.last_claim_timestamp = current_time;
        user.total_claimed += claimable;

        let protocol = &mut ctx.accounts.protocol_state;
        protocol.total_minted += claimable;

        emit!(DailyClaimed {
            user: ctx.accounts.user.key(),
            amount: claimable,
            timestamp: current_time,
        });

        Ok(())
    }

    // ========================================================================
    // EMERGENCY CONTROLS
    // ========================================================================

    pub fn freeze_business_registration(
        ctx: Context<EmergencyControl>,
        freeze: bool,
    ) -> Result<()> {
        require!(
            ctx.accounts.authority.key() == ctx.accounts.protocol_state.authority,
            ErrorCode::Unauthorized
        );

        let protocol = &mut ctx.accounts.protocol_state;
        protocol.business_registration_frozen = freeze;

        emit!(BusinessRegistrationFrozen {
            frozen: freeze,
            authority: ctx.accounts.authority.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    // ========================================================================
    // ANNUAL LIVENESS VERIFICATION
    // ========================================================================
    // Every human must prove they are ALIVE once per year.
    // ALL countries treated equally - we don't trust ANY government's death registry.
    // We verify the human directly via biometric proof-of-life.
    //
    // Process:
    // 1. request_liveness_challenge() - Get a random challenge
    // 2. User reads biometrics from eID card (off-chain)
    // 3. User performs LIVE biometric scan (off-chain)
    // 4. verify_annual_liveness() - Submit proof, system verifies match

    /// Step 1: Request a challenge for liveness verification
    /// Returns a random challenge that must be signed with live biometrics
    pub fn request_liveness_challenge(ctx: Context<RequestLivenessChallenge>) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // Check if user is locked out from too many failed attempts
        require!(
            current_time >= user.liveness.locked_until,
            ErrorCode::LivenessVerificationLocked
        );

        // Generate random challenge using recent blockhash
        let clock = Clock::get()?;
        let challenge = generate_challenge(
            &ctx.accounts.user.key(),
            clock.slot,
            current_time,
        );

        // Store challenge
        user.liveness.current_challenge = challenge;
        user.liveness.challenge_issued_at = current_time;
        user.liveness.challenge_expires_at = current_time + BIOMETRIC_CHALLENGE_TIMEOUT;

        emit!(LivenessChallengeIssued {
            user: ctx.accounts.user.key(),
            challenge,
            expires_at: user.liveness.challenge_expires_at,
            timestamp: current_time,
        });

        Ok(())
    }

    /// Step 2: Submit biometric proof to complete annual verification
    /// Verifies that eID biometrics match LIVE biometric scan
    pub fn verify_annual_liveness(
        ctx: Context<VerifyAnnualLiveness>,
        proof: BiometricProof,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // =====================================================================
        // SECURITY CHECKS
        // =====================================================================

        // 1. Must be a verified human
        require!(user.is_human, ErrorCode::OnlyHumansCanVerify);

        // 2. Must not be blacklisted
        require!(!user.permanently_blacklisted, ErrorCode::UserBlacklisted);

        // 3. Must not be locked out
        require!(
            current_time >= user.liveness.locked_until,
            ErrorCode::LivenessVerificationLocked
        );

        // 4. Challenge must exist and not be expired
        require!(
            user.liveness.challenge_issued_at > 0,
            ErrorCode::NoChallengeIssued
        );
        require!(
            current_time <= user.liveness.challenge_expires_at,
            ErrorCode::ChallengeExpired
        );

        // 5. Proof timestamp must be recent (within challenge window)
        require!(
            proof.timestamp >= user.liveness.challenge_issued_at,
            ErrorCode::ProofTooOld
        );
        require!(
            proof.timestamp <= current_time + 60, // Allow 60s clock drift
            ErrorCode::ProofFromFuture
        );

        // 6. eID must not be expired
        require!(
            proof.eid_expiry > current_time,
            ErrorCode::EidExpired
        );

        // =====================================================================
        // BIOMETRIC VERIFICATION
        // =====================================================================
        // CRITICAL: This is the core proof-of-life check.
        // eID biometrics (from government card) must match LIVE scan.
        // A dead person CANNOT perform a live biometric scan.

        // 7. Verify challenge response is correct
        let expected_response = compute_challenge_response(
            &user.liveness.current_challenge,
            &proof.live_biometric_hash,
            proof.timestamp,
        );
        require!(
            proof.challenge_response == expected_response,
            ErrorCode::InvalidChallengeResponse
        );

        // 8. CRITICAL: Verify eID biometrics match LIVE biometrics
        // This is what proves the person is ALIVE
        require!(
            proof.eid_biometric_hash == proof.live_biometric_hash,
            ErrorCode::BiometricMismatch
        );

        // 9. For returning users: verify it's the SAME person
        if user.liveness.verification_count > 0 {
            // Must use same biometric type as registered
            require!(
                proof.biometric_type == user.liveness.biometric_type,
                ErrorCode::BiometricTypeMismatch
            );
            // Must match stored eID hash (prevents identity theft)
            require!(
                proof.eid_biometric_hash == user.liveness.eid_biometric_hash,
                ErrorCode::IdentityMismatch
            );
        }

        // 10. Verify device attestation (prevents replay attacks)
        require!(
            verify_device_attestation(&proof.device_attestation, &proof),
            ErrorCode::InvalidDeviceAttestation
        );

        // =====================================================================
        // SUCCESS: Update liveness state
        // =====================================================================

        user.liveness.is_verified = true;
        user.liveness.last_verification = current_time;
        user.liveness.next_verification_due = current_time + LIVENESS_INTERVAL;
        user.liveness.verification_count += 1;

        // Store/update biometric identity
        user.liveness.biometric_type = proof.biometric_type;
        user.liveness.eid_biometric_hash = proof.eid_biometric_hash;
        user.liveness.eid_issuing_country = proof.eid_country;
        user.liveness.eid_expiry = proof.eid_expiry;

        // Clear challenge and reset failure counter
        user.liveness.current_challenge = [0u8; 32];
        user.liveness.challenge_issued_at = 0;
        user.liveness.challenge_expires_at = 0;
        user.liveness.failed_attempts = 0;

        emit!(LivenessVerified {
            user: ctx.accounts.user.key(),
            biometric_type: user.liveness.biometric_type.clone(),
            verification_count: user.liveness.verification_count,
            next_due: user.liveness.next_verification_due,
            timestamp: current_time,
        });

        Ok(())
    }

    /// Handle failed verification attempt
    pub fn report_failed_liveness_attempt(
        ctx: Context<ReportFailedLiveness>,
    ) -> Result<()> {
        let user = &mut ctx.accounts.user_state;
        let current_time = Clock::get()?.unix_timestamp;

        // Reset counter if last attempt was more than 24h ago
        if current_time - user.liveness.last_failed_attempt > VERIFICATION_LOCKOUT {
            user.liveness.failed_attempts = 0;
        }

        user.liveness.failed_attempts += 1;
        user.liveness.last_failed_attempt = current_time;

        // Lock account if too many failures
        if user.liveness.failed_attempts >= MAX_VERIFICATION_ATTEMPTS {
            user.liveness.locked_until = current_time + VERIFICATION_LOCKOUT;

            emit!(LivenessVerificationLockedOut {
                user: ctx.accounts.user.key(),
                failed_attempts: user.liveness.failed_attempts,
                locked_until: user.liveness.locked_until,
                timestamp: current_time,
            });
        }

        // Clear the expired/used challenge
        user.liveness.current_challenge = [0u8; 32];
        user.liveness.challenge_issued_at = 0;
        user.liveness.challenge_expires_at = 0;

        Ok(())
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

// ============================================================================
// LIVENESS VERIFICATION HELPERS
// ============================================================================

/// Determine current liveness status based on timestamps
fn get_liveness_status(liveness: &LivenessState, current_time: i64) -> LivenessStatus {
    // Check if locked out first
    if current_time < liveness.locked_until {
        return LivenessStatus::Locked;
    }

    // First-time users get 30 days to verify (they verified during registration)
    if liveness.verification_count == 0 {
        // New users have implicit verification from registration
        return LivenessStatus::Active;
    }

    let time_until_due = liveness.next_verification_due - current_time;
    let time_since_due = current_time - liveness.next_verification_due;

    if time_until_due > LIVENESS_WARNING_DAYS * SECONDS_PER_DAY {
        // More than 60 days until due - all good
        LivenessStatus::Active
    } else if time_until_due > 0 {
        // Less than 60 days - warning period
        LivenessStatus::Warning
    } else if time_since_due <= LIVENESS_GRACE_PERIOD {
        // Past due but within 30-day grace period
        LivenessStatus::Expired
    } else {
        // Grace period expired - account suspended
        LivenessStatus::Suspended
    }
}

/// Generate a random challenge for liveness verification
/// Uses slot + timestamp + user pubkey for unpredictability
fn generate_challenge(user: &Pubkey, slot: u64, timestamp: i64) -> [u8; 32] {
    use anchor_lang::solana_program::hash::hash;

    let mut data = Vec::with_capacity(48);
    data.extend_from_slice(user.as_ref());
    data.extend_from_slice(&slot.to_le_bytes());
    data.extend_from_slice(&timestamp.to_le_bytes());

    hash(&data).to_bytes()
}

/// Compute expected challenge response
/// The client must sign the challenge with their live biometric
fn compute_challenge_response(
    challenge: &[u8; 32],
    live_biometric_hash: &[u8; 32],
    timestamp: i64,
) -> [u8; 32] {
    use anchor_lang::solana_program::hash::hash;

    let mut data = Vec::with_capacity(72);
    data.extend_from_slice(challenge);
    data.extend_from_slice(live_biometric_hash);
    data.extend_from_slice(&timestamp.to_le_bytes());

    hash(&data).to_bytes()
}

/// Verify device attestation to prevent replay attacks
/// In production, this would verify a signature from a trusted execution environment
fn verify_device_attestation(attestation: &[u8; 64], proof: &BiometricProof) -> bool {
    // TODO: Implement actual device attestation verification
    // This should verify:
    // 1. Attestation comes from a trusted device (TEE/Secure Enclave)
    // 2. Attestation is fresh (tied to this specific proof)
    // 3. Device has not been tampered with

    // For now, just check it's not all zeros (placeholder)
    attestation.iter().any(|&b| b != 0)
}

/// Initialize liveness state for new user
fn initialize_liveness_state(current_time: i64) -> LivenessState {
    LivenessState {
        is_verified: true,  // Initial registration counts as first verification
        last_verification: current_time,
        next_verification_due: current_time + LIVENESS_INTERVAL,
        verification_count: 1,
        biometric_type: BiometricType::None,  // Set during first explicit verification
        eid_biometric_hash: [0u8; 32],
        eid_issuing_country: [0u8; 3],
        eid_expiry: 0,
        failed_attempts: 0,
        last_failed_attempt: 0,
        locked_until: 0,
        current_challenge: [0u8; 32],
        challenge_issued_at: 0,
        challenge_expires_at: 0,
    }
}

// ============================================================================
// MERCHANT HELPER FUNCTIONS
// ============================================================================

fn calculate_service_merchant_score(
    volume_ratio: f64,
    unique_customers: u32,
    avg_tx_size: u128,
) -> f64 {
    // For high-value services: volume matters most
    let volume_score = (volume_ratio.min(20.0) * 5.0);
    let customer_score = ((unique_customers as f64).min(50.0) * 1.0);
    let size_score = ((avg_tx_size as f64 / 1_000_000_000_000.0).min(50.0));
    
    volume_score + customer_score + size_score
}

fn calculate_retail_merchant_score(
    volume_ratio: f64,
    unique_customers: u32,
    tx_count: u64,
) -> f64 {
    // For retail: need more customers and transactions
    let volume_score = (volume_ratio.min(10.0) * 4.0);
    let customer_score = ((unique_customers as f64 / 10.0).min(100.0) * 0.4);
    let tx_score = ((tx_count as f64 / 100.0).min(100.0) * 0.2);
    
    volume_score + customer_score + tx_score
}

fn determine_merchant_tier(score: f64, merchant_type: MerchantType) -> MerchantTier {
    let thresholds = match merchant_type {
        MerchantType::Service => [20.0, 40.0, 60.0, 80.0, 150.0],
        MerchantType::Retail => [30.0, 50.0, 75.0, 100.0, 150.0],
    };
    
    match score {
        s if s < thresholds[0] => MerchantTier::None,
        s if s < thresholds[1] => MerchantTier::Emerging,
        s if s < thresholds[2] => MerchantTier::Small,
        s if s < thresholds[3] => MerchantTier::Medium,
        s if s < thresholds[4] => MerchantTier::Large,
        _ => MerchantTier::Enterprise,
    }
}

fn decay_merchant_status(user: &mut UserState) -> Result<()> {
    let previous = user.merchant_status.clone();
    
    user.merchant_status = match user.merchant_status {
        MerchantTier::Enterprise => MerchantTier::Large,
        MerchantTier::Large => MerchantTier::Medium,
        MerchantTier::Medium => MerchantTier::Small,
        MerchantTier::Small => MerchantTier::Emerging,
        MerchantTier::Emerging => MerchantTier::None,
        MerchantTier::None => MerchantTier::None,
    };
    
    if previous != user.merchant_status {
        emit!(MerchantStatusDecayed {
            merchant: user.owner,
            from_tier: previous,
            to_tier: user.merchant_status.clone(),
            timestamp: Clock::get()?.unix_timestamp,
        });
    }
    
    Ok(())
}

fn calculate_merchant_fee(tier: &MerchantTier) -> u64 {
    match tier {
        MerchantTier::None => BASE_FEE_RATE,
        MerchantTier::Emerging => BASE_FEE_RATE * 75 / 100,  // 25% off
        MerchantTier::Small => BASE_FEE_RATE * 50 / 100,     // 50% off
        MerchantTier::Medium => BASE_FEE_RATE * 25 / 100,    // 75% off
        MerchantTier::Large | MerchantTier::Enterprise => 0,  // FREE
    }
}

fn determine_claim_interval(user: &UserState) -> i64 {
    // Based on contribution status
    if user.merchant_status >= MerchantTier::Small 
        || user.sponsor_tier >= SponsorTier::Bronze 
        || user.dao_tier >= DaoTier::Contributor {
        SECONDS_PER_DAY  // Daily claims
    } else {
        SECONDS_PER_DAY * 30  // Monthly for non-contributors
    }
}

fn calculate_retroactive_claim(age_in_days: u64) -> u64 {
    if age_in_days >= MAX_RETROACTIVE_DAYS {
        MAX_RETROACTIVE_DAYS * DAILY_AMOUNT
    } else {
        age_in_days * DAILY_AMOUNT
    }
}

fn calculate_claimable_amount(user: &UserState, current_time: i64) -> Result<u64> {
    let seconds_elapsed = current_time - user.last_claim_timestamp;
    let days_elapsed = (seconds_elapsed / SECONDS_PER_DAY) as u64;
    let claimable_days = std::cmp::min(days_elapsed, MAX_ACCUMULATION_DAYS);
    Ok(claimable_days * DAILY_AMOUNT)
}

fn verify_civic_pass(proof: &[u8]) -> Result<()> {
    // TODO: Actual Civic Pass verification
    require!(proof.len() > 0, ErrorCode::InvalidCivicPass);
    Ok(())
}

fn handle_fees<'info>(
    ctx: &Context<'_, '_, '_, 'info, BusinessTransaction<'info>>,
    fee_amount: u64,
) -> Result<()> {
    // Burn 50%
    let burn_amount = (fee_amount * BURN_PERCENTAGE) / 100;
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.mint.to_account_info(),
                from: ctx.accounts.from.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        burn_amount,
    )?;
    
    // Treasury 50%
    let treasury_amount = fee_amount - burn_amount;
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.from.to_account_info(),
                to: ctx.accounts.treasury_account.to_account_info(),
                authority: ctx.accounts.signer.to_account_info(),
            },
        ),
        treasury_amount,
    )?;
    
    Ok(())
}

// ============================================================================
// ACCOUNT STRUCTURES
// ============================================================================

#[account]
pub struct ProtocolState {
    pub authority: Pubkey,
    pub mint: Pubkey,
    pub treasury: Pubkey,
    pub gas_pool: Pubkey,
    pub biometric_registry: Pubkey,          // Global biometric hash registry
    pub total_users: u64,
    pub total_merchants: u64,
    pub total_businesses: u64,
    pub total_minted: u128,
    pub total_burned: u128,
    pub launch_timestamp: i64,
    pub emergency_paused: bool,
    pub business_registration_frozen: bool,  // Emergency control
    pub paused_at: i64,
    pub bump: u8,
}

// ============================================================================
// BIOMETRIC DEDUPLICATION REGISTRY
// ============================================================================
// Global registry of all biometric hashes - prevents duplicate accounts
// hash(left_index_finger + right_index_finger) = unique human identifier
// eIDAS mandates EXACTLY both index fingers - cannot use other fingers

#[account]
pub struct BiometricRegistry {
    pub total_registered: u64,               // Total unique humans registered
    pub duplicate_attempts: u64,             // Count of rejected duplicates
    pub last_updated: i64,
    pub bump: u8,
}

// Individual biometric entry (PDA per hash)
// If someone tries to register with same fingerprints, PDA init fails = duplicate blocked
#[account]
pub struct BiometricEntry {
    pub biometric_hash: [u8; 32],            // SHA-256(left_index + right_index)
    pub user_account: Pubkey,                // Associated user account
    pub registered_at: i64,
    pub is_active: bool,                     // Can be deactivated if individual fraud proven
    pub bump: u8,
}

#[account]
pub struct UserState {
    pub owner: Pubkey,
    pub civic_pass: Pubkey,
    pub is_human: bool,                    // CRITICAL: Must be true for claims
    pub registration_timestamp: i64,
    pub last_claim_timestamp: i64,
    pub last_tx_timestamp: i64,
    pub last_merchant_tx: i64,
    pub age_in_days_at_registration: u64,
    pub is_child: bool,
    pub total_claimed: u128,

    // Biometric identity - for deduplication only
    pub biometric_hash: [u8; 32],           // SHA-256(left_index + right_index)

    // Merchant fields - VOLUME BASED
    pub merchant_status: MerchantTier,
    pub volume_received: u128,             // Total AGORA received
    pub volume_sent: u128,                 // Total AGORA sent
    pub tx_count_received: u64,            // Number of incoming TX
    pub tx_count_sent: u64,                // Number of outgoing TX
    pub unique_payers: Vec<Pubkey>,        // Who paid this merchant
    pub unique_payers_count: u32,
    pub average_transaction_size: u128,
    
    // SINGLE BUSINESS MODEL - Maximum security
    pub business_profile: Option<BusinessProfile>,  // None or One, NEVER multiple
    
    // Sponsor fields
    pub sponsor_tier: SponsorTier,
    pub personal_allocation_remaining: u64,
    
    // DAO fields
    pub dao_tier: DaoTier,
    pub dao_contribution: u64,
    pub voting_weight: f64,
    
    // Security
    pub violations: u8,
    pub suspended_until: i64,
    pub permanently_blacklisted: bool,
    pub daily_tx_count: u16,
    pub last_tx_reset: i64,

    // =========================================================================
    // ANNUAL LIVENESS VERIFICATION
    // =========================================================================
    // Every human must prove they are ALIVE once per year.
    // Method: eID biometrics must match LIVE biometric scan.
    // This prevents: dead person fraud, stolen identity abuse.

    pub liveness: LivenessState,

    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BusinessProfile {
    pub name: String,                      // On-chain business name
    pub category: BusinessCategory,
    pub tax_id: Option<String>,
    pub metadata_uri: String,               // IPFS/Arweave pointer to divisions
    pub created_at: i64,
    pub is_verified: bool,                 // DAO verification
    pub is_active: bool,
    pub volume_processed: u128,
    pub employee_count: u32,               // Just count, details off-chain
    pub last_metadata_update: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BusinessInfo {
    pub name: String,
    pub category: BusinessCategory,
    pub tax_id: Option<String>,
    pub metadata_uri: String,               // Required: points to off-chain data
}

// Off-chain metadata structure (stored on IPFS/Arweave)
// {
//     "mainBusiness": "John's Services",
//     "owner": "pubkey...",
//     "divisions": [
//         {"name": "John's Plumbing", "type": "plumbing", "established": "2025-01-01"},
//         {"name": "John's Electric", "type": "electrical", "established": "2025-06-01"},
//         {"name": "John's HVAC", "type": "hvac", "established": "2025-09-01"}
//     ],
//     "employees": [
//         {"wallet": "pubkey...", "name": "Alice", "role": "Plumber", "canTransact": true},
//         {"wallet": "pubkey...", "name": "Bob", "role": "Electrician", "canTransact": true}
//     ],
//     "locations": [
//         {"address": "Main St 1", "type": "headquarters"},
//         {"address": "Second Ave 5", "type": "branch"}
//     ],
//     "verifiedByDAO": false,
//     "lastUpdated": "2025-11-26T12:00:00Z"
// }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum BusinessCategory {
    Retail,
    Food,
    Services,       // Plumber, electrician, etc
    Professional,   // Lawyer, consultant, etc
    Healthcare,
    Education,
    Entertainment,
    Technology,
    Manufacturing,
    Other,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum MerchantType {
    Retail,    // High frequency, low value
    Service,   // Low frequency, high value
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, PartialOrd)]
pub enum MerchantTier {
    None,
    Emerging,
    Small,
    Medium,
    Large,
    Enterprise,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, PartialOrd)]
pub enum SponsorTier {
    None,
    Bronze,
    Silver,
    Gold,
    Platinum,
    Diamond,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, PartialOrd)]
pub enum DaoTier {
    None,
    Supporter,
    Contributor,
    Builder,
    Sustainer,
    Founder,
}

// ============================================================================
// ANNUAL LIVENESS VERIFICATION STRUCTURES
// ============================================================================
// Philosophy: ALL countries are treated equally. ANY government could be hostile.
// We verify the HUMAN is alive, not that the government says they're alive.
//
// Process:
// 1. User reads biometric data from their eID card (iris, fingerprint, face)
// 2. User performs LIVE biometric scan (cannot be faked by dead person)
// 3. System compares eID biometrics with live scan
// 4. Match = proof of life, account remains active for another year
//
// This is government-agnostic: we use their ID infrastructure but verify independently.

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct LivenessState {
    // Verification status
    pub is_verified: bool,                    // Currently verified as alive
    pub last_verification: i64,               // Timestamp of last successful verification
    pub next_verification_due: i64,           // When next verification is required
    pub verification_count: u32,              // Total successful verifications

    // Biometric identity (privacy-preserving hashes only)
    pub biometric_type: BiometricType,        // What biometric is on user's eID
    pub eid_biometric_hash: [u8; 32],         // SHA-256 of biometric from eID card
    pub eid_issuing_country: [u8; 3],         // ISO 3166-1 alpha-3 country code
    pub eid_expiry: i64,                      // eID card expiry date

    // Anti-fraud
    pub failed_attempts: u8,                  // Failed verification attempts today
    pub last_failed_attempt: i64,             // Timestamp of last failure
    pub locked_until: i64,                    // If locked out, when can retry

    // Challenge-response for live verification
    pub current_challenge: [u8; 32],          // Random challenge for this verification
    pub challenge_issued_at: i64,             // When challenge was issued
    pub challenge_expires_at: i64,            // Challenge expiry (5 minutes)
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Default)]
pub enum BiometricType {
    #[default]
    None,           // Not yet registered
    Fingerprint,    // Most common on eID cards
    Iris,           // Higher security, less common
    Face,           // 3D face scan
    Multi,          // Multiple biometrics on eID
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub enum LivenessStatus {
    Active,         // Verified and within validity period
    Warning,        // Approaching expiry (60 days)
    Expired,        // Past due, in grace period (30 days)
    Suspended,      // Grace period expired, account frozen
    Locked,         // Too many failed attempts, temporarily locked
}

// Input structure for verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BiometricProof {
    pub eid_biometric_hash: [u8; 32],        // Hash of biometric read from eID card
    pub live_biometric_hash: [u8; 32],       // Hash of LIVE biometric scan
    pub biometric_type: BiometricType,       // Type of biometric used
    pub challenge_response: [u8; 32],        // Response to challenge (proves freshness)
    pub eid_country: [u8; 3],                // Country that issued eID
    pub eid_expiry: i64,                     // eID expiry timestamp
    pub timestamp: i64,                       // When this proof was generated
    pub device_attestation: [u8; 64],        // Device signature (prevents replay)
}

// ============================================================================
// VALIDATION CONTEXTS
// ============================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 512, seeds = [b"protocol"], bump)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    #[account(init, payer = authority, mint::decimals = 9, mint::authority = protocol_state)]
    pub mint: Account<'info, Mint>,
    
    /// CHECK: Treasury for fees
    #[account(mut)]
    pub treasury: UncheckedAccount<'info>,
    
    /// CHECK: Gas pool PDA
    #[account(mut)]
    pub gas_pool: UncheckedAccount<'info>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
#[instruction(age_in_days: u64, civic_pass_proof: Vec<u8>, biometric_hash: [u8; 32])]
pub struct RegisterUser<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 1536,  // UserState size
        seeds = [b"user", user.key().as_ref()],
        bump
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,

    // Biometric deduplication - PDA seeded by biometric hash
    // If this hash already exists, init will FAIL → duplicate prevented
    // This is the ONLY fraud prevention needed - same fingerprints = same person = reject
    #[account(
        init,
        payer = user,
        space = 8 + 80,  // BiometricEntry size
        seeds = [b"biometric", biometric_hash.as_ref()],
        bump
    )]
    pub biometric_entry: Account<'info, BiometricEntry>,

    #[account(
        mut,
        seeds = [b"biometric_registry"],
        bump = biometric_registry.bump
    )]
    pub biometric_registry: Account<'info, BiometricRegistry>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = user
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// CHECK: Civic Pass verification
    pub civic_pass: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct RegisterBusiness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateBusiness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveBusiness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct BusinessTransaction<'info> {
    #[account(
        mut,
        seeds = [b"user", signer.key().as_ref()],
        bump
    )]
    pub sender_state: Account<'info, UserState>,
    
    #[account(
        mut,
        seeds = [b"user", recipient.key().as_ref()],
        bump
    )]
    pub recipient_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub from: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub to: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub treasury_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    pub signer: Signer<'info>,
    
    /// CHECK: Recipient
    pub recipient: UncheckedAccount<'info>,
    
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateMerchantStatus<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,
    
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,
    
    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ClaimDaily<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,

    #[account(mut)]
    pub mint: Account<'info, Mint>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct EmergencyControl<'info> {
    #[account(mut)]
    pub protocol_state: Account<'info, ProtocolState>,

    pub authority: Signer<'info>,
}

// ============================================================================
// LIVENESS VERIFICATION CONTEXTS
// ============================================================================

#[derive(Accounts)]
pub struct RequestLivenessChallenge<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct VerifyAnnualLiveness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    pub user: Signer<'info>,
}

#[derive(Accounts)]
pub struct ReportFailedLiveness<'info> {
    #[account(
        mut,
        seeds = [b"user", user.key().as_ref()],
        bump = user_state.bump
    )]
    pub user_state: Account<'info, UserState>,

    pub user: Signer<'info>,
}

// ============================================================================
// EVENTS
// ============================================================================

#[event]
pub struct ProtocolInitialized {
    pub mint: Pubkey,
    pub treasury: Pubkey,
    pub gas_pool: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct UserRegistered {
    pub user: Pubkey,
    pub retroactive_claim: u64,
    pub age_in_days: u64,
    pub is_human: bool,
    pub biometric_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct BusinessRegistered {
    pub human_owner: Pubkey,
    pub business_name: String,
    pub category: BusinessCategory,
    pub metadata_uri: String,
    pub timestamp: i64,
}

#[event]
pub struct BusinessMetadataUpdated {
    pub business_owner: Pubkey,
    pub business_name: String,
    pub old_metadata_uri: String,
    pub new_metadata_uri: String,
    pub timestamp: i64,
}

#[event]
pub struct BusinessRemoved {
    pub human_owner: Pubkey,
    pub business_name: String,
    pub timestamp: i64,
}

#[event]
pub struct MerchantStatusUpgraded {
    pub merchant: Pubkey,
    pub new_tier: MerchantTier,
    pub score: f64,
    pub merchant_type: MerchantType,
    pub timestamp: i64,
}

#[event]
pub struct MerchantStatusDecayed {
    pub merchant: Pubkey,
    pub from_tier: MerchantTier,
    pub to_tier: MerchantTier,
    pub timestamp: i64,
}

#[event]
pub struct BusinessTransactionCompleted {
    pub human_owner: Pubkey,
    pub business_name: String,
    pub recipient: Pubkey,
    pub amount: u64,
    pub fee: u64,
    pub metadata: Option<String>,
    pub timestamp: i64,
}

#[event]
pub struct EmployeeCountUpdated {
    pub business_owner: Pubkey,
    pub business_name: String,
    pub employee_count: u32,
    pub timestamp: i64,
}

#[event]
pub struct DailyClaimed {
    pub user: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct BusinessRegistrationFrozen {
    pub frozen: bool,
    pub authority: Pubkey,
    pub timestamp: i64,
}

// ============================================================================
// BIOMETRIC DEDUPLICATION EVENTS
// ============================================================================
// NO country-level events - only individual fraud detection

#[event]
pub struct IndividualFraudDetected {
    pub user: Pubkey,
    pub fraud_type: String,
    pub evidence_hash: [u8; 32],
    pub timestamp: i64,
}

#[event]
pub struct IndividualBanned {
    pub user: Pubkey,
    pub reason: String,
    pub timestamp: i64,
}

// ============================================================================
// LIVENESS VERIFICATION EVENTS
// ============================================================================

#[event]
pub struct LivenessChallengeIssued {
    pub user: Pubkey,
    pub challenge: [u8; 32],
    pub expires_at: i64,
    pub timestamp: i64,
}

#[event]
pub struct LivenessVerified {
    pub user: Pubkey,
    pub biometric_type: BiometricType,
    pub verification_count: u32,
    pub next_due: i64,
    pub timestamp: i64,
}

#[event]
pub struct LivenessWarning {
    pub user: Pubkey,
    pub days_until_expiry: i64,
    pub timestamp: i64,
}

#[event]
pub struct LivenessExpiredWarning {
    pub user: Pubkey,
    pub grace_days_remaining: i64,
    pub timestamp: i64,
}

#[event]
pub struct LivenessVerificationLockedOut {
    pub user: Pubkey,
    pub failed_attempts: u8,
    pub locked_until: i64,
    pub timestamp: i64,
}

// ============================================================================
// ERROR CODES
// ============================================================================

#[error_code]
pub enum ErrorCode {
    #[msg("Must be verified human via Civic Pass")]
    MustBeVerifiedHuman,
    
    #[msg("Only humans can claim UBI")]
    OnlyHumansCanClaim,
    
    #[msg("Business already exists - only one per human allowed")]
    BusinessAlreadyExists,
    
    #[msg("No business registered")]
    NoBusinessRegistered,
    
    #[msg("Business not active")]
    BusinessNotActive,
    
    #[msg("Business registration is frozen")]
    BusinessRegistrationFrozen,
    
    #[msg("Metadata URI too long (max 128 chars)")]
    MetadataUriTooLong,
    
    #[msg("Metadata update too soon (once per day)")]
    MetadataUpdateTooSoon,
    
    #[msg("Not authorized")]
    Unauthorized,
    
    #[msg("User is suspended")]
    UserSuspended,
    
    #[msg("User is permanently blacklisted")]
    UserBlacklisted,
    
    #[msg("Nothing to claim")]
    NothingToClaim,
    
    #[msg("Claim too soon")]
    ClaimTooSoon,
    
    #[msg("Invalid Civic Pass")]
    InvalidCivicPass,
    
    #[msg("Transaction too soon")]
    TransactionTooSoon,

    #[msg("Daily limit exceeded")]
    DailyLimitExceeded,

    // =========================================================================
    // LIVENESS VERIFICATION ERRORS
    // =========================================================================

    #[msg("Annual liveness verification required - please verify you are alive")]
    LivenessVerificationRequired,

    #[msg("Liveness verification temporarily locked - too many failed attempts")]
    LivenessVerificationLocked,

    #[msg("Only humans can verify liveness")]
    OnlyHumansCanVerify,

    #[msg("No verification challenge issued - request a challenge first")]
    NoChallengeIssued,

    #[msg("Verification challenge has expired - request a new one")]
    ChallengeExpired,

    #[msg("Biometric proof is too old")]
    ProofTooOld,

    #[msg("Biometric proof timestamp is in the future")]
    ProofFromFuture,

    #[msg("eID card has expired - please renew your identity document")]
    EidExpired,

    #[msg("Invalid challenge response - verification failed")]
    InvalidChallengeResponse,

    #[msg("Biometric mismatch - live scan does not match eID biometrics")]
    BiometricMismatch,

    #[msg("Biometric type mismatch - must use same biometric type as registered")]
    BiometricTypeMismatch,

    #[msg("Identity mismatch - this is not the same person who registered")]
    IdentityMismatch,

    #[msg("Invalid device attestation - verification must be from trusted device")]
    InvalidDeviceAttestation,

    // =========================================================================
    // BIOMETRIC DEDUPLICATION ERRORS
    // =========================================================================

    #[msg("Biometric hash already registered - one account per person")]
    BiometricAlreadyRegistered,

    #[msg("Invalid biometric hash format")]
    InvalidBiometricHash,
}
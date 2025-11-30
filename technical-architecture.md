# AGORA Protocol - Technical Architecture
## Complete System Specification

**Version 2.0 (Dual-Program Architecture)**  
**November 2025**  
**Status: Smart Contracts Complete, Audit Pending**

---

## Executive Summary

AGORA Protocol implements Universal Basic Income on Solana blockchain through a **dual-program architecture**:

1. **AGORA Core (IMMUTABLE)** - Sacred promise of 100 AGORA/day, locked forever
2. **AGORA Governance (UPGRADEABLE)** - DAO, sanctions, fees, managed by community

This separation ensures the fundamental UBI promise can never be changed, while allowing democratic governance of everything else.

**Key Innovation:** Immutable economic rights encoded in code that cannot be modified by anyone.

---

## Table of Contents

1. [Dual-Program Architecture](#1-dual-program-architecture)
2. [AGORA Core (Immutable Program)](#2-agora-core-immutable-program)
3. [AGORA Governance (Upgradeable Program)](#3-agora-governance-upgradeable-program)
4. [Inter-Program Communication](#4-inter-program-communication)
5. [Token Economics](#5-token-economics)
6. [Identity & Biometrics](#6-identity--biometrics)
7. [DAO Governance](#7-dao-governance)
8. [Security Model](#8-security-model)
9. [Deployment Process](#9-deployment-process)
10. [Technical Requirements](#10-technical-requirements)

---

## 1. Dual-Program Architecture

### 1.1 Why Two Programs?

**The Problem:**

Traditional smart contracts face an impossible choice:
- **Immutable** = Cannot fix bugs, cannot add features, cannot adapt
- **Upgradeable** = Governance can change anything, including fundamental promises

**The Solution:**

Split into two programs with different properties:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           AGORA PROTOCOL                                    │
├─────────────────────────────────┬───────────────────────────────────────────┤
│                                 │                                           │
│    AGORA CORE                   │    AGORA GOVERNANCE                       │
│    (IMMUTABLE)                  │    (UPGRADEABLE)                          │
│                                 │                                           │
│    • 100 AGORA/day             │    • DAO proposals                        │
│    • User registration          │    • Country sanctions                    │
│    • Biometric deduplication    │    • Fee parameters                       │
│    • Daily claims               │    • Gas pool                             │
│    • Liveness verification      │    • Merchant detection                   │
│    • Biometric updates          │    • Treasury management                  │
│    • Basic transfers            │    • Reputation system                    │
│                                 │                                           │
│    LOCKED FOREVER               │    DAO CAN UPGRADE (67% vote)             │
│                                 │                                           │
└─────────────────────────────────┴───────────────────────────────────────────┘
```

### 1.2 Program Identifiers

```rust
// AGORA Core - IMMUTABLE
declare_id!("AGoRACoreXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

// AGORA Governance - UPGRADEABLE
declare_id!("AGoRAGovXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");
```

### 1.3 Directory Structure

```
smart-contracts/
├── manifesto.md                    # Philosophy and vision
├── technical-architecture.md       # This document
│
└── programs/
    ├── Anchor.toml                 # Anchor configuration
    ├── Cargo.toml                  # Workspace configuration
    │
    ├── agora-core/                 # IMMUTABLE PROGRAM
    │   ├── Cargo.toml
    │   └── src/
    │       └── lib.rs              # 1,531 lines
    │
    └── agora-governance/           # UPGRADEABLE PROGRAM
        ├── Cargo.toml
        └── src/
            └── lib.rs              # 1,685 lines
```

---

## 2. AGORA Core (Immutable Program)

### 2.1 Purpose

AGORA Core contains everything that must NEVER change:

- The sacred 100 AGORA/day promise
- User identity verification
- Token minting logic
- Biometric deduplication

**After deployment, upgrade authority is permanently revoked:**
```bash
solana program set-upgrade-authority <AGORA_CORE_ID> --final
```

This command is IRREVERSIBLE. No one can modify Core after this.

### 2.2 The Sacred Constant

```rust
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
// IMMUTABLE - SACRED CONSTANT - DO NOT MODIFY UNDER ANY CIRCUMSTANCES
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
//
// This value is the IDENTITY of AGORA Protocol. It must NEVER change.
// No DAO vote, no governance proposal, no emergency, no upgrade can modify this.
// Changing this value would destroy the protocol's fundamental promise.
// 100 AGORA per day, per human, forever. This is non-negotiable.
//
// !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

pub const DAILY_AMOUNT: u64 = 100_000_000_000;  // 100 AGORA (9 decimals) - IMMUTABLE FOREVER
```

### 2.3 Core Constants

```rust
/// Maximum retroactive days for initial claim (1 year)
pub const MAX_RETROACTIVE_DAYS: u64 = 365;

/// Maximum days tokens can accumulate before expiring
pub const MAX_ACCUMULATION_DAYS: u64 = 30;

/// Seconds in a day
pub const SECONDS_PER_DAY: i64 = 86400;

/// Token decimals (Solana standard)
pub const TOKEN_DECIMALS: u8 = 9;

/// Minimum time between transactions (rate limiting)
pub const MIN_TIME_BETWEEN_TX: i64 = 60;

/// Age threshold for child protection (18 years)
pub const CHILD_AGE_THRESHOLD: u64 = 6570;

/// Liveness verification period (1 year)
pub const LIVENESS_PERIOD_SECONDS: i64 = 31_536_000;

/// Grace period after liveness expiry (30 days)
pub const LIVENESS_GRACE_PERIOD: i64 = 2_592_000;

/// Governance program ID (for reading sanction accounts)
pub const GOVERNANCE_PROGRAM_ID: Pubkey = pubkey!("AGoRAGovXXX...");
```

### 2.4 Core Functions

#### 2.4.1 initialize()
```rust
pub fn initialize(ctx: Context<Initialize>) -> Result<()>
```

**Purpose:** Initialize protocol state, token mint, and biometric registry.

**Called:** Once at deployment.

**Creates:**
- ProtocolState PDA
- BiometricRegistry PDA
- AGORA token mint

#### 2.4.2 register_user()
```rust
pub fn register_user(
    ctx: Context<RegisterUser>,
    age_in_days: u64,
    biometric_hash: [u8; 32],
    citizenship: [u8; 3],
) -> Result<()>
```

**Purpose:** Register new user with biometric verification.

**Process:**
1. Verify biometric hash is unique (not already registered)
2. Create UserState PDA
3. Create BiometricRecord PDA
4. Calculate retroactive claim (up to 365 days)
5. Check for country sanctions (read from Governance)
6. Mint initial claim (reduced if sanctioned)

**Sybil Resistance:**
- Biometric hash derived from eID fingerprint/iris
- Same biometric = same hash, always
- Cannot create multiple accounts

#### 2.4.3 claim_daily()
```rust
pub fn claim_daily(ctx: Context<ClaimDaily>) -> Result<()>
```

**Purpose:** Claim accumulated daily UBI.

**Process:**
1. Verify user is registered and verified
2. Check liveness is valid (or in grace period)
3. Calculate days since last claim (max 30)
4. Check for country sanctions
5. Calculate actual amount: `DAILY_AMOUNT × days × sanction_pct / 100`
6. Mint tokens to user

**Key Formula:**
```rust
let base_amount = claimable_days * DAILY_AMOUNT;  // Always 100 AGORA/day
let actual_amount = base_amount * sanction_pct / 100;  // Reduced if sanctioned
```

#### 2.4.4 verify_liveness()
```rust
pub fn verify_liveness(
    ctx: Context<VerifyLiveness>,
    live_biometric_hash: [u8; 32],
) -> Result<()>
```

**Purpose:** Annual verification that user is still alive.

**Process:**
1. User provides LIVE biometric scan
2. System computes hash of live scan
3. Compare with stored hash - must match
4. If match: extend liveness for 1 year

**Security:**
- Prevents dead person fraud
- Prevents identity theft (thief cannot produce matching biometrics)
- Required annually

#### 2.4.5 update_biometric()
```rust
pub fn update_biometric(
    ctx: Context<UpdateBiometric>,
    old_biometric_proof: [u8; 32],
    new_biometric_hash: [u8; 32],
) -> Result<()>
```

**Purpose:** Update biometric when technology changes (e.g., fingerprint → DNA).

**Process:**
1. Verify ownership of OLD biometric (live scan must match)
2. Verify NEW biometric is unique (not already registered)
3. Invalidate old BiometricRecord
4. Create new BiometricRecord
5. Reset liveness verification

**Security:**
- Requires proof of BOTH old AND new biometric
- Attacker would need physical access to both
- Prevents account theft

**Use Cases:**
- eID upgrades from fingerprint to iris
- Country adopts DNA-based identity
- Technology improvements

#### 2.4.6 transfer_tokens()
```rust
pub fn transfer_tokens(
    ctx: Context<TransferTokens>,
    amount: u64,
) -> Result<()>
```

**Purpose:** Transfer AGORA between users.

**Process:**
1. Verify sender is verified and not a child
2. Check rate limiting (60 seconds between TX)
3. Read fee parameters from Governance
4. Calculate fees (base rate × activity multiplier)
5. Transfer to recipient
6. Send treasury portion to treasury
7. Burn fee portion

### 2.5 Core Account Structures

#### ProtocolState
```rust
pub struct ProtocolState {
    pub authority: Pubkey,       // Original deployer (reference only)
    pub mint: Pubkey,            // AGORA token mint
    pub treasury: Pubkey,        // Protocol treasury
    pub total_users: u64,        // Registered user count
    pub total_minted: u128,      // Total ever minted
    pub total_burned: u128,      // Total ever burned
    pub launch_timestamp: i64,   // Protocol launch time
    pub is_initialized: bool,
    pub bump: u8,
}
```

#### UserState
```rust
pub struct UserState {
    pub owner: Pubkey,
    pub registration_timestamp: i64,
    pub last_claim_timestamp: i64,
    pub age_in_days_at_registration: u64,
    pub citizenship: [u8; 3],          // ISO 3166-1 alpha-3
    pub is_verified: bool,
    pub is_child: bool,                // Under 18
    pub total_claimed: u128,
    pub locked_balance: u64,           // For children
    pub transaction_count: u64,
    pub last_transaction_timestamp: i64,
    pub liveness_verified_at: i64,
    pub liveness_expires_at: i64,
    pub bump: u8,
}
```

#### BiometricRecord
```rust
pub struct BiometricRecord {
    pub hash: [u8; 32],                // SHA-256 of biometric
    pub user: Pubkey,
    pub registered_at: i64,
    pub is_registered: bool,           // False if superseded
    pub invalidated_at: i64,           // When invalidated (0 if active)
    pub previous_hash: Option<[u8; 32]>, // For tracking updates
    pub bump: u8,
}
```

### 2.6 Core Events

```rust
ProtocolInitialized { authority, mint, treasury, launch_timestamp }
UserRegistered { user, registration_timestamp, age_in_days, citizenship, is_child }
InitialClaimMinted { user, retroactive_days, amount, sanction_percentage, is_locked }
DailyClaimed { user, days_claimed, base_amount, sanction_percentage, actual_amount, is_locked, timestamp }
LivenessVerified { user, verified_at, expires_at }
BiometricUpdated { user, old_hash, new_hash, timestamp }
TokensTransferred { sender, recipient, amount, fee_amount, burn_amount, treasury_amount, timestamp }
```

### 2.7 Core Errors

```rust
BiometricAlreadyRegistered  // Same biometric already exists
UserNotVerified             // User not verified
LivenessExpired             // Must verify liveness
BiometricMismatch           // Live scan doesn't match stored
NothingToClaim              // No days accumulated
ChildCannotTransfer         // Under 18 cannot transfer
InvalidAmount               // Amount must be > 0
RateLimitExceeded           // Too fast between transactions
Unauthorized                // Not account owner
BiometricNotRegistered      // Biometric record invalid
```

---

## 3. AGORA Governance (Upgradeable Program)

### 3.1 Purpose

AGORA Governance contains everything that MAY need to change:

- DAO proposal system
- Country sanctions
- Fee parameters
- Gas pool management
- Merchant detection

**The DAO can upgrade this program via Constitutional proposal (75% supermajority).**

### 3.2 Governance Constants

All parameters are DAO-adjustable. These are initial values:

#### Proposal Bonds
```rust
pub const PROPOSAL_BOND_STANDARD: u64 = 20_000_000_000_000;       // 20,000 AGORA
pub const PROPOSAL_BOND_TREASURY: u64 = 50_000_000_000_000;       // 50,000 AGORA
pub const PROPOSAL_BOND_CONSTITUTIONAL: u64 = 100_000_000_000_000; // 100,000 AGORA
pub const PROPOSAL_BOND_SANCTION: u64 = 75_000_000_000_000;       // 75,000 AGORA
```

#### Dynamic Quorums (percentage of registered users)
```rust
// Quorum percentages
pub const QUORUM_PCT_STANDARD: u64 = 100;        // 1% of users
pub const QUORUM_PCT_TREASURY: u64 = 200;        // 2% of users
pub const QUORUM_PCT_SANCTION: u64 = 500;        // 5% of users
pub const QUORUM_PCT_CONSTITUTIONAL: u64 = 1000; // 10% of users

// Minimum quorums (equivalent to 1M users - no changes until protocol established)
pub const QUORUM_MIN_STANDARD: u64 = 10_000;       // 1% of 1M
pub const QUORUM_MIN_TREASURY: u64 = 20_000;       // 2% of 1M
pub const QUORUM_MIN_SANCTION: u64 = 50_000;       // 5% of 1M
pub const QUORUM_MIN_CONSTITUTIONAL: u64 = 100_000; // 10% of 1M

// No maximum - democracy scales infinitely
// Formula: max(MIN, total_users * PCT / 10000)
```

**Example quorums at different user counts:**

| Users | Standard (1%) | Treasury (2%) | Sanction (5%) | Constitutional (10%) |
|-------|---------------|---------------|---------------|----------------------|
| 100,000 | 10,000 (min) | 20,000 (min) | 50,000 (min) | 100,000 (min) |
| 500,000 | 10,000 (min) | 20,000 (min) | 50,000 (min) | 100,000 (min) |
| 1,000,000 | 10,000 | 20,000 | 50,000 | 100,000 |
| 10,000,000 | 100,000 | 200,000 | 500,000 | 1,000,000 |
| 100,000,000 | 1,000,000 | 2,000,000 | 5,000,000 | 10,000,000 |

**Note:** No governance changes possible until 1M+ users (minimums enforce this).

#### Approval Thresholds (basis points)
```rust
pub const APPROVAL_STANDARD: u64 = 5001;       // >50%
pub const APPROVAL_TREASURY: u64 = 5001;       // >50%
pub const APPROVAL_SANCTION: u64 = 6700;       // >67%
pub const APPROVAL_CONSTITUTIONAL: u64 = 7500; // >75%
```

#### Reputation System
```rust
pub const REP_PROPOSAL_PASSED: i32 = 2;     // Passed
pub const REP_PROPOSAL_REJECTED: i32 = 1;   // Rejected but quorum reached
pub const REP_NO_QUORUM_50: i32 = -1;       // 50%+ of quorum
pub const REP_NO_QUORUM_25: i32 = -2;       // 25-50% of quorum
pub const REP_NO_QUORUM_10: i32 = -3;       // <25% of quorum (spam)
pub const REP_THRESHOLD_BAN: i32 = -10;     // Banned at -10
```

#### Fee Parameters
```rust
pub const DEFAULT_BASE_FEE_RATE: u64 = 5;         // 0.05%
pub const DEFAULT_BURN_PERCENTAGE: u64 = 50;      // 50% burned
pub const ACTIVE_MULTIPLIER: u64 = 80;            // 0.8x
pub const NORMAL_MULTIPLIER: u64 = 100;           // 1.0x
pub const INACTIVE_MULTIPLIER: u64 = 150;         // 1.5x
pub const DORMANT_MULTIPLIER: u64 = 200;          // 2.0x
```

### 3.3 Governance Functions

#### 3.3.1 initialize()
```rust
pub fn initialize(ctx: Context<InitializeGovernance>) -> Result<()>
```

Initialize governance state, fee parameters, and proposal registry.

#### 3.3.2 create_proposal()
```rust
pub fn create_proposal(
    ctx: Context<CreateProposal>,
    proposal_type: ProposalType,
    title: [u8; 64],
    description_hash: [u8; 32],
    treasury_amount: u64,
    treasury_recipient: Pubkey,
    sanction_country: [u8; 3],
    sanction_ubi_pct: u8,
) -> Result<()>
```

**Process:**
1. Check proposer reputation (banned at -10)
2. Calculate bond with multiplier: `bond × (1 + abs(reputation) / 2)`
3. Transfer bond to escrow
4. Create proposal with voting period

#### 3.3.3 vote_on_proposal()
```rust
pub fn vote_on_proposal(
    ctx: Context<VoteOnProposal>,
    choice: u8,  // 0=No, 1=Yes, 2=Abstain
) -> Result<()>
```

**Rules:**
- 1 person = 1 vote (not token-weighted)
- Cannot change vote once cast
- Must be verified user

#### 3.3.4 finalize_proposal()
```rust
pub fn finalize_proposal(ctx: Context<FinalizeProposal>) -> Result<()>
```

**Process:**
1. Check voting period ended
2. Calculate quorum and approval
3. Determine outcome and reputation change
4. Return or forfeit bond
5. Execute if passed

#### 3.3.5 impose_sanction()
```rust
pub fn impose_sanction(ctx: Context<ImposeSanction>) -> Result<()>
```

Called after Sanction proposal passes. Creates CountrySanction account.

#### 3.3.6 lift_sanction()
```rust
pub fn lift_sanction(ctx: Context<LiftSanction>) -> Result<()>
```

Called after lift proposal passes. Deactivates sanction.

#### 3.3.7 sponsor_gas_pool()
```rust
pub fn sponsor_gas_pool(ctx: Context<SponsorGasPool>, amount: u64) -> Result<()>
```

Contribute SOL to gas pool. Receive tier benefits.

#### 3.3.8 update_fee_parameters()
```rust
pub fn update_fee_parameters(
    ctx: Context<UpdateFeeParameters>,
    new_base_rate: u64,
    new_burn_pct: u64,
) -> Result<()>
```

Called after Constitutional proposal passes.

### 3.4 Governance Account Structures

#### CountrySanction (READ BY CORE)
```rust
pub struct CountrySanction {
    pub country_code: [u8; 3],    // ISO 3166-1 alpha-3
    pub ubi_percentage: u8,       // 1-99 (never 0 or 100)
    pub imposed_at: i64,
    pub expires_at: i64,
    pub proposal_id: u64,
    pub is_active: bool,
    pub lifted_early: bool,
    pub lifted_at: i64,
    pub bump: u8,
}
```

**Critical:** Core reads this account to determine UBI reduction.

#### FeeState (READ BY CORE)
```rust
pub struct FeeState {
    pub base_fee_rate: u64,       // In basis points
    pub burn_percentage: u64,     // 0-100
    pub active_multiplier: u64,   // 80 = 0.8x
    pub normal_multiplier: u64,   // 100 = 1.0x
    pub inactive_multiplier: u64, // 150 = 1.5x
    pub dormant_multiplier: u64,  // 200 = 2.0x
    pub last_updated: i64,
    pub bump: u8,
}
```

#### Proposal
```rust
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
```

#### ProposerState
```rust
pub struct ProposerState {
    pub user: Pubkey,
    pub proposal_reputation: i32,  // Can be negative
    pub proposals_created: u32,
    pub proposals_passed: u32,
    pub proposals_rejected: u32,
    pub proposals_expired: u32,
    pub total_votes_cast: u64,
    pub bump: u8,
}
```

---

## 4. Inter-Program Communication

### 4.1 Design Principle

**Core READS from Governance. Governance CANNOT MODIFY Core.**

```
┌─────────────────────┐         ┌─────────────────────┐
│                     │         │                     │
│    AGORA CORE       │ ──────► │  AGORA GOVERNANCE   │
│                     │  reads  │                     │
│                     │         │                     │
└─────────────────────┘         └─────────────────────┘
        │                                │
        │ CANNOT call                    │ CANNOT modify
        │ functions                      │ Core state
        ▼                                ▼
```

### 4.2 What Core Reads

**CountrySanction accounts:**
```rust
// In claim_daily()
let sanction_pct = match &ctx.accounts.country_sanction {
    Some(sanction_account) => {
        // Deserialize and check
        let data = sanction_account.try_borrow_data()?;
        if is_active && valid_format {
            sanction.ubi_percentage as u64
        } else {
            100  // Not active = full UBI
        }
    },
    None => 100,  // No sanction = full 100%
};
```

**FeeState account:**
```rust
// In transfer_tokens()
let (fee_rate, burn_pct) = match &ctx.accounts.fee_state {
    Some(fee_account) => {
        // Deserialize fee parameters
        (fee_state.base_fee_rate, fee_state.burn_percentage)
    },
    None => (5, 50),  // Defaults if not found
};
```

### 4.3 Account Verification

Core verifies Governance accounts by PDA seeds:

```rust
// CountrySanction PDA
#[account(
    seeds = [b"sanction", citizenship.as_ref()],
    bump,
    seeds::program = GOVERNANCE_PROGRAM_ID,
)]
pub country_sanction: Option<AccountInfo<'info>>,
```

This ensures Core only reads legitimate accounts from Governance program.

### 4.4 Security Implications

**What Governance CAN do:**
- Create sanction accounts (reduce UBI to 1-99%)
- Modify fee parameters
- Upgrade Governance program code

**What Governance CANNOT do:**
- Change DAILY_AMOUNT constant in Core
- Modify Core's mint logic
- Set sanction to 0% (minimum is 1%)
- Override Core's biometric verification
- Call any Core function

**Even with 100% vote, DAO cannot touch Core's sacred promise.**

---

## 5. Token Economics

### 5.1 Supply Model

```
Total Supply = Σ(minted) - Σ(burned)

where:
  minted = users × 100 AGORA/day × sanction_pct
  burned = transfer_volume × fee_rate × burn_pct
```

**No maximum supply cap.** Supply grows with users, shrinks with activity.

### 5.2 Fee Structure

| User Status | Condition | Multiplier | Effective Fee |
|-------------|-----------|------------|---------------|
| Active | TX within 7 days | 0.8x | 0.04% |
| Normal | TX within 30 days | 1.0x | 0.05% |
| Inactive | TX within 90 days | 1.5x | 0.075% |
| Dormant | No TX for 90+ days | 2.0x | 0.10% |

**Fee Distribution:**
- 50% burned (deflationary)
- 50% to treasury (operations)

### 5.3 Merchant Tiers

| Tier | Min Customers OR Min Volume | Fee Discount |
|------|---------------------------|--------------|
| Emerging | 10 customers OR 1,000 AGORA | 25% off |
| Small | 25 customers OR 10,000 AGORA | 50% off |
| Medium | 100 customers OR 50,000 AGORA | 75% off |
| Large | 500 customers OR 100,000 AGORA | FREE |
| Enterprise | 2,000 customers OR 500,000 AGORA | FREE |

---

## 6. Identity & Biometrics

### 6.1 Registration Flow

```
User with eID
     │
     ▼
┌─────────────────┐
│ 1. eID Reader   │ ──► Extracts fingerprint/iris
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 2. SHA-256 Hash │ ──► Deterministic: same biometric = same hash
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ 3. Check Unique │ ──► BiometricRecord PDA exists? REJECT
└────────┬────────┘
         │ (unique)
         ▼
┌─────────────────┐
│ 4. Register     │ ──► Create UserState + BiometricRecord
└────────┬────────┘
         │
         ▼
    User Verified
```

### 6.2 Biometric Update Flow

```
User wants to update (e.g., fingerprint → DNA)
     │
     ▼
┌─────────────────────┐
│ 1. Prove OLD        │ ──► Live scan of OLD biometric
│    biometric        │     Must match stored hash
└─────────┬───────────┘
          │ (match)
          ▼
┌─────────────────────┐
│ 2. Provide NEW      │ ──► New hash from updated eID
│    biometric        │     Must NOT exist already
└─────────┬───────────┘
          │ (unique)
          ▼
┌─────────────────────┐
│ 3. Update records   │ ──► Old record invalidated
│                     │     New record created
└─────────┬───────────┘
          │
          ▼
    Biometric Updated
```

### 6.3 Security Analysis

**Attack: Create multiple accounts**
- Blocked by: Biometric hash uniqueness check
- Same fingerprint = same hash = rejected

**Attack: Use someone else's eID**
- Blocked by: Live scan must match stored hash
- Thief's fingerprint ≠ victim's fingerprint

**Attack: Steal account via biometric update**
- Blocked by: Must prove BOTH old AND new biometric
- Would need physical access to both biometrics

**Attack: Dead person fraud**
- Blocked by: Annual liveness verification
- Dead person cannot provide live scan

---

## 7. DAO Governance

### 7.1 Proposal Types

| Type | Quorum | Approval | Bond | Period | Use |
|------|--------|----------|------|--------|-----|
| Standard | 1% of users | >50% | 20,000 AGORA | 3 days | Minor changes |
| Treasury | 2% of users | >50% | 50,000 AGORA | 7 days | Spend funds |
| Sanction | 5% of users | >67% | 75,000 AGORA | 14 days | Country sanctions |
| Constitutional | 10% of users | >75% | 100,000 AGORA | 14 days | Major changes |

**Note:** Quorums have min/max bounds. See Section 3.2 for details.

### 7.2 Voting Rules

- **1 person = 1 vote** (not token-weighted)
- Vote once per proposal (cannot change)
- Options: Yes (1), No (0), Abstain (2)
- Only verified users can vote

### 7.3 Reputation System

**Automatic calculation based on proposal outcomes:**

| Outcome | Reputation Change |
|---------|-------------------|
| Passed | +2 |
| Rejected (with quorum) | +1 |
| 50%+ of quorum | -1 |
| 25-50% of quorum | -2 |
| <25% of quorum (spam) | -3 |

**At -10 reputation:** Banned from creating proposals.

**Bond multiplier:** `bond × (1 + abs(reputation) / 2)`
- At -4 reputation: bond × 3
- At -8 reputation: bond × 5

### 7.4 Sanction Limits

**Sanctions can ONLY:**
- Reduce UBI percentage (1-99%)
- Apply to specific country codes
- Last up to 365 days
- Be lifted early by new vote

**Sanctions can NEVER:**
- Reduce to 0% (minimum is 1%)
- Apply to individuals (only countries)
- Override Core's minting logic

---

## 8. Security Model

### 8.1 Immutability Guarantees

**Core Program:**
```bash
# After deployment
solana program set-upgrade-authority <CORE_ID> --final
```

This command:
- Permanently removes upgrade authority
- Cannot be reversed
- No one can modify Core ever again

### 8.2 Governance Upgrade Process

1. Constitutional proposal created (100,000 AGORA bond)
2. 14-day voting period
3. 10% of users quorum required
4. 75% supermajority required
5. If passed: multi-sig executes upgrade
6. 7-day timelock before activation

### 8.3 Attack Resistance

| Attack | Protection |
|--------|------------|
| Sybil (multiple accounts) | Biometric deduplication |
| Dead person fraud | Annual liveness verification |
| Account theft | Dual biometric proof for updates |
| DAO takeover | Core immutable, sanctions limited to 1% |
| Flash loan voting | 1 person = 1 vote, not tokens |
| Spam proposals | High bonds, reputation system |

### 8.4 Bug Scenarios

**Bug in Core:**
- Core is immutable, cannot fix
- Deploy new Core, migrate users
- Old Core remains (broken but immutable)
- This is the cost of true immutability

**Bug in Governance:**
- DAO votes to upgrade Governance
- Bug is fixed in new version
- System continues

---

## 9. Deployment Process

### 9.1 Phase 1: Development
- [x] Core program development (1,531 lines)
- [x] Governance program development (1,685 lines)
- [ ] Integration tests
- [ ] Devnet deployment

### 9.2 Phase 2: Audit
- [ ] Internal code review
- [ ] Community bug bounty (5-200 SOL)
- [ ] Professional audit (OtterSec/Neodyme)
- [ ] Fix discovered issues

### 9.3 Phase 3: Mainnet Launch

**Step 1: Deploy Governance**
```bash
anchor deploy --program-name agora_governance
# Keep upgrade authority for potential fixes
```

**Step 2: Deploy Core**
```bash
anchor deploy --program-name agora_core
```

**Step 3: Initialize Programs**
```bash
# Initialize Governance
anchor run init_governance

# Initialize Core (links to Governance)
anchor run init_core
```

**Step 4: LOCK CORE FOREVER**
```bash
# THIS IS IRREVERSIBLE
solana program set-upgrade-authority <AGORA_CORE_ID> --final
```

After Step 4:
- Core is IMMUTABLE forever
- 100 AGORA/day is guaranteed forever
- No one can change it

---

## 10. Technical Requirements

### 10.1 Infrastructure

**RPC Nodes:**
- Primary: GenesysGo or Helius
- Backup: Public Solana RPC
- Estimated: ~2 SOL/month

**Frontend:**
- GitHub Pages (free)
- CloudFlare CDN

**Storage:**
- On-chain: User accounts, governance
- IPFS: Proposal descriptions

### 10.2 Transaction Costs

| Operation | Cost |
|-----------|------|
| User registration | ~0.01 SOL |
| Daily claim | ~0.001 SOL |
| Transfer | ~0.00025 SOL |
| Create proposal | ~0.002 SOL |
| Vote | ~0.0005 SOL |

### 10.3 Account Storage

| Account | Size | Rent |
|---------|------|------|
| UserState | ~200 bytes | ~0.002 SOL |
| BiometricRecord | ~120 bytes | ~0.001 SOL |
| Proposal | ~300 bytes | ~0.003 SOL |
| VoteRecord | ~60 bytes | ~0.0006 SOL |

---

## Appendix A: Error Codes

### Core Errors
```rust
BiometricAlreadyRegistered  // 6000
UserNotVerified             // 6001
LivenessExpired             // 6002
BiometricMismatch           // 6003
NothingToClaim              // 6004
ChildCannotTransfer         // 6005
InvalidAmount               // 6006
RateLimitExceeded           // 6007
Unauthorized                // 6008
BiometricNotRegistered      // 6009
```

### Governance Errors
```rust
ProposerBanned              // 6100
InvalidSanctionPercentage   // 6101
VoterNotVerified            // 6102
ProposalNotActive           // 6103
VotingEnded                 // 6104
InvalidVoteChoice           // 6105
AlreadyVoted                // 6106
VotingNotEnded              // 6107
ProposalNotPassed           // 6108
WrongProposalType           // 6109
GasPoolPaused               // 6110
InvalidAmount               // 6111
FeeTooHigh                  // 6112
InvalidBurnPercentage       // 6113
```

---

## Appendix B: Mathematical Constants

```rust
// Child protection ratios
CHILD_ACCESSIBLE_RATIO = 1/(π×e) ≈ 0.1169  // ~11.69%
TEEN_ACCESSIBLE_RATIO = φ/(π×e) ≈ 0.1892   // ~18.92%

// Treasury mint rate
TREASURY_RATE = 1/(π×e) ≈ 0.00117  // ~0.117% per claim
```

---

## Version History

- **v1.0.0** (Nov 2025): Single program architecture
- **v2.0.0** (Nov 2025): **Dual-program architecture**
  - Split into Core (immutable) and Governance (upgradeable)
  - Added update_biometric function
  - Added sanction limits (minimum 1%)
  - Clear separation of powers

---

**For technical questions:** GitHub Issues  
**For security concerns:** security@agora-protocol.org

---

*"Code is law, but some laws must be immutable."*

**END OF DOCUMENT**

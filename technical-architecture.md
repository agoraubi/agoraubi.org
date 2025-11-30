# AGORA Protocol - Technical Architecture
## Complete System Specification

**Version 1.5.0**  
**November 2025**  
**Status: DAO Governance Implemented (lib.rs v3.6)**

---

## Executive Summary

AGORA Protocol implements Universal Basic Income on Solana blockchain, distributing 100 AGORA tokens daily to every verified human. The system uses Civic Pass for identity verification, implements retroactive fairness for early adopters, and includes a self-balancing economic model with personalized burn rates.

**Key Innovation:** Money that cannot be bought - only earned through humanity or economic activity.

**Implementation Status (v3.6):**
- ✅ Core token distribution
- ✅ User registration & biometric deduplication  
- ✅ Merchant auto-detection
- ✅ Activity-based fees
- ✅ Annual liveness verification
- ✅ Gas Pool system
- ✅ Treasury Mint system
- ✅ **DAO Governance system (NEW)** — proposals, voting, bonds, reputation
- ✅ **Country sanctions system (NEW)** — democratic response to atrocities
- ⏳ Civic Pass integration (placeholder)

---

## Table of Contents

1. [System Overview](#1-system-overview)
2. [Token Economics](#2-token-economics)
3. [Smart Contract Architecture](#3-smart-contract-architecture)
4. [Identity Verification](#4-identity-verification)
5. [Distribution Mechanism](#5-distribution-mechanism)
6. [Inflation Model](#6-inflation-model)
7. [Governance Structure](#7-governance-structure)
8. [Gas Pool Economics & Security](#8-gas-pool-economics--security)
9. [Merchant Economy & Auto-Detection](#9-merchant-economy--auto-detection)
10. [DAO Proposal System](#10-dao-proposal-system) ✅ NEW
11. [Country Sanctions](#11-country-sanctions) ✅ NEW
12. [Security Considerations](#12-security-considerations)
13. [Implementation Roadmap](#13-implementation-roadmap)
14. [Technical Requirements](#14-technical-requirements)

---

## 1. System Overview

### Core Components

```
â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”
â”‚              AGORA Protocol                  â”‚
â”œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”¤
â”‚                                               â”‚
â”‚  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”  â”Œâ”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”  â”‚
â”‚  â”‚  Token   â”‚  â”‚ Identity â”‚  â”‚   DAO    â”‚  â”‚
â”‚  â”‚  Minting â”‚â—„â”€â”¤  Verify  â”‚  â”‚ Govern   â”‚  â”‚
â”‚  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜  â”‚
â”‚       â–²              â–²             â–²         â”‚
â”‚       â”‚              â”‚             â”‚         â”‚
â”‚  â”Œâ”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”´â”€â”€â”€â”€â”   â”‚
â”‚  â”‚        Solana Blockchain              â”‚   â”‚
â”‚  â”‚   Program ID: AGORA11...111           â”‚   â”‚
â”‚  â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜   â”‚
â”‚                                               â”‚
â””â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”˜
```

### Technology Stack

- **Blockchain:** Solana
- **Smart Contracts:** Rust with Anchor Framework
- **Token Standard:** SPL Token with Token Extensions
- **Identity:** Civic Pass Integration
- **Frontend:** Pure HTML/CSS/JavaScript
- **Treasury Dashboard:** Real-time on-chain data

---

## 2. Token Economics

### 2.1 Distribution Model

#### Initial Claims (Retroactive Fairness)
```
Age-based distribution:
â”œâ”€â”€ Adults (â‰¥365 days old): 36,500 AGORA (1 year retroactive)
â”œâ”€â”€ Children (<365 days old): age_in_days Ã— 100 AGORA
â””â”€â”€ Maximum accumulation: 30 days (3,000 AGORA)
```

#### Daily Distribution
- **Amount:** 100 AGORA per verified human per day
- **Frequency:** Claimable anytime (accumulates up to 30 days)
- **Requirement:** Valid Civic Pass verification

### 2.2 Supply Dynamics

```rust
// No maximum supply cap
total_supply = Î£(daily_mint) + Î£(initial_claims) - Î£(burned_tokens)

where:
  daily_mint = verified_users Ã— 100 Ã— days_elapsed
  initial_claims = Î£(retroactive_distributions)
  burned_tokens = transfer_volume Ã— burn_rate Ã— 0.5
```

### 2.3 Activity-Based Fee System

**Base Fee:** 0.05% (5 basis points)

Fees are determined by user's recent activity level. This encourages using AGORA as currency rather than hoarding.

#### Regular Users (Non-Merchants)

| Activity Status | Condition | Multiplier | Effective Fee |
|-----------------|-----------|------------|---------------|
| Active | TX within 7 days | 0.8x | 0.04% |
| Normal | TX within 30 days | 1.0x | 0.05% |
| Inactive | TX within 90 days | 1.5x | 0.075% |
| Dormant | No TX for 90+ days | 2.0x | 0.10% |

#### Merchants (Based on Tier)

| Merchant Tier | Fee Discount | Effective Fee |
|---------------|--------------|---------------|
| Emerging | 25% off | 0.0375% |
| Small | 50% off | 0.025% |
| Medium | 75% off | 0.0125% |
| Large | FREE | 0% |
| Enterprise | FREE | 0% |

**Note:** Merchant fee always takes precedence over activity-based fee (merchants get the better rate).

#### Fee Distribution
- 50% burned (deflationary pressure)
- 50% to protocol treasury (operations)

#### DAO Adjustable Parameters

All thresholds are DAO-adjustable via governance:
- Activity day thresholds (7, 30, 90 days)
- Fee multipliers (0.8x, 1.0x, 1.5x, 2.0x)
- Merchant volume thresholds
- Base fee rate

**Why DAO-adjustable?** Real economy will determine actual prices. We don't know yet if a haircut will cost 500 or 5,000 AGORA. DAO must tune thresholds based on observed transaction patterns.

---

## 3. Smart Contract Architecture

### 3.1 Core Program Structure

```rust
// Program ID (Mainnet)
declare_id!("AGORA111111111111111111111111111111111111111");

// Main Contract Modules
pub mod agora_protocol {
    mod token_mint;      // Token creation and minting
    mod user_registry;   // User registration and verification
    mod distribution;    // Daily and retroactive claims
    mod governance;      // DAO voting mechanisms
    mod treasury;        // Fee collection and management
    mod burn;           // Personalized burn rate calculation
}
```

### 3.2 Account Structure

#### Protocol State (PDA)
```rust
pub struct ProtocolState {
    pub authority: Pubkey,           // DAO authority
    pub mint: Pubkey,               // AGORA token mint
    pub treasury: Pubkey,           // Protocol treasury
    pub total_users: u64,           // Registered user count
    pub total_minted: u128,         // Total tokens minted
    pub total_burned: u128,         // Total tokens burned
    pub launch_timestamp: i64,      // Protocol launch time
    pub civic_gateway: Pubkey,      // Civic Pass gateway
    pub bump: u8,                   // PDA bump seed
}
```

#### User Account (PDA)
```rust
pub struct UserState {
    pub is_initialized: bool,
    pub owner: Pubkey,                      // Wallet address
    pub civic_pass: Pubkey,                 // Civic Pass ID
    pub registration_timestamp: i64,
    pub last_claim_timestamp: i64,
    pub total_claimed: u128,
    pub age_in_days_at_registration: u64,
    pub is_child: bool,                     // Under 18
    pub is_merchant: bool,                  // Verified merchant
    pub transaction_count: u64,             // For behavior scoring
    pub last_transaction_timestamp: i64,
    pub hoarding_score: u8,                 // 0-100 scale
    pub spending_frequency: u8,             // 0-100 scale
    pub bump: u8,
}
```

### 3.3 Core Instructions

#### 1. Initialize Protocol
```rust
pub fn initialize(ctx: Context<Initialize>) -> Result<()>
```

#### 2. Register User
```rust
pub fn register_user(
    ctx: Context<RegisterUser>,
    age_in_days: u64,
    civic_pass_proof: Vec<u8>
) -> Result<()>
```

#### 3. Claim Daily UBI
```rust
pub fn claim_daily(ctx: Context<ClaimDaily>) -> Result<()>
```

#### 4. Transfer with Burn
```rust
pub fn transfer_with_burn(
    ctx: Context<TransferWithBurn>,
    amount: u64
) -> Result<()>
```

---

## 4. Identity Verification

### 4.1 Civic Pass Integration

**Status:** Placeholder implementation in `lib.rs` - Production integration required before mainnet.

#### Overview

Civic Pass is the leading decentralized identity verification solution on Solana, with over 2 million verifications processed. AGORA uses Civic Pass combined with biometric deduplication for maximum Sybil resistance.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    AGORA IDENTITY VERIFICATION ARCHITECTURE                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐                   │
│  │   Layer 1    │    │   Layer 2    │    │   Layer 3    │                   │
│  │  Civic Pass  │ +  │  Biometric   │ +  │   Liveness   │ = Maximum         │
│  │ (Gov ID+KYC) │    │    Hash      │    │   (Annual)   │   Security        │
│  └──────────────┘    └──────────────┘    └──────────────┘                   │
│        │                    │                   │                            │
│        ▼                    ▼                   ▼                            │
│  "Real identity       "Unique person      "Currently                        │
│   document exists"     (fingerprints)"     alive"                           │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Registration Flow

```
User Registration Flow (Detailed):

1. USER CONNECTS WALLET
   └─► Phantom, Solflare, Backpack, etc.

2. FRONTEND CHECKS FOR CIVIC PASS
   └─► findGatewayToken(connection, wallet, gatekeeperNetwork)
   └─► If found & valid: Skip to step 5

3. NO PASS → CIVIC VERIFICATION
   └─► requestGatewayToken() opens Civic modal
   └─► User uploads government ID (passport, national ID)
   └─► User takes live selfie (liveness check)
   └─► Civic AI verifies ID matches selfie
   └─► ~2-5 minutes verification time

4. CIVIC ISSUES GATEWAY TOKEN
   └─► On-chain, non-transferable token
   └─► Contains: owner_wallet, gatekeeper_network, state, expire_time
   └─► Personal data stored OFF-chain (privacy preserving)

5. USER INITIATES REGISTRATION
   └─► Provides biometric hash (fingerprint template)
   └─► Calls register_user(age_in_days, civic_proof, biometric_hash)

6. SMART CONTRACT VERIFICATION
   └─► Verify Gateway Token owner == user.key()
   └─► Verify gatekeeper_network == AGORA approved network
   └─► Verify token state == Active
   └─► Verify not expired
   └─► Verify biometric hash not already registered (PDA check)

7. ACCOUNT CREATION
   └─► UserState PDA initialized
   └─► BiometricEntry PDA initialized (deduplication)
   └─► Retroactive claim minted
   └─► User is now verified AGORA participant
```

#### Gatekeeper Networks (Pass Types)

| Network ID | Type | Use Case | AGORA Status |
|------------|------|----------|--------------|
| `ignREusXmGrscGNUesoU9mxfds9AiYTezUKex2PsZV6` | CAPTCHA | Bot resistance | ❌ Not sufficient |
| `tgnuXXNMDLK8dy7Xm1TdeGyc95MDym4bvAQCwcW21Bf` | Uniqueness | Sybil resistance | ⚠️ Partial |
| `bni1ewus6aMxTxBi5SAfzEmmXLf8KcVFRmTfproJuKw` | ID Verification | KYC compliance | ⚠️ Partial |
| `gatbGF9DvLAw3kWyn1EmH5Nh1Sqp8sTukF7yaQpSc71` | **ID + Liveness** | Full verification | ✅ **RECOMMENDED** |

**AGORA Requirement:** ID + Liveness (`gatbGF9DvLAw...`) ensures:
- Government-issued ID verified
- Live selfie matches ID photo
- Real person, real identity, currently alive

#### Smart Contract Integration

**Current Placeholder (`lib.rs`):**
```rust
fn verify_civic_pass(proof: &[u8]) -> Result<()> {
    // PLACEHOLDER - only checks non-empty
    require!(proof.len() > 0, ErrorCode::InvalidCivicPass);
    Ok(())
}
```

**Production Implementation:**
```rust
use solana_gateway::state::{GatewayToken, GatewayTokenState};

// Add to RegisterUser accounts
pub struct RegisterUser<'info> {
    // ... existing accounts ...
    
    /// Gateway token (Civic Pass) - must be valid and not expired
    #[account(
        constraint = gateway_token.owner_wallet == user.key() 
            @ ErrorCode::GatewayTokenOwnerMismatch,
        constraint = gateway_token.gatekeeper_network == protocol_state.civic_gatekeeper_network 
            @ ErrorCode::InvalidGatekeeperNetwork,
    )]
    pub gateway_token: Account<'info, GatewayToken>,
}

fn verify_civic_pass_production(
    gateway_token: &Account<GatewayToken>,
    user: &Pubkey,
    gatekeeper_network: &Pubkey,
    current_time: i64,
) -> Result<()> {
    // 1. Verify owner matches
    require!(
        gateway_token.owner_wallet == *user,
        ErrorCode::GatewayTokenOwnerMismatch
    );
    
    // 2. Verify correct gatekeeper network
    require!(
        gateway_token.gatekeeper_network == *gatekeeper_network,
        ErrorCode::InvalidGatekeeperNetwork
    );
    
    // 3. Verify token is active
    require!(
        gateway_token.state == GatewayTokenState::Active,
        ErrorCode::GatewayTokenNotActive
    );
    
    // 4. Verify not expired
    if let Some(expiry) = gateway_token.expire_time {
        require!(current_time < expiry, ErrorCode::GatewayTokenExpired);
    }
    
    Ok(())
}
```

#### Frontend Integration

**Dependencies:**
```json
{
  "dependencies": {
    "@civic/solana-gateway-react": "^0.9.0",
    "@identity.com/solana-gateway-ts": "^0.11.0"
  }
}
```

**React Implementation:**
```typescript
import { GatewayProvider, useGateway, IdentityButton } from "@civic/solana-gateway-react";
import { findGatewayToken } from "@identity.com/solana-gateway-ts";

// AGORA's approved gatekeeper network (ID + Liveness)
const AGORA_GATEKEEPER_NETWORK = new PublicKey("gatbGF9DvLAw3kWyn1EmH5Nh1Sqp8sTukF7yaQpSc71");

// Wrap app in GatewayProvider
function App() {
  return (
    <GatewayProvider
      wallet={wallet}
      gatekeeperNetwork={AGORA_GATEKEEPER_NETWORK}
      connection={connection}
      cluster="mainnet-beta"
    >
      <RegistrationFlow />
    </GatewayProvider>
  );
}

// Registration component
function RegistrationFlow() {
  const { gatewayToken, gatewayStatus, requestGatewayToken } = useGateway();
  
  // Check if user already has valid pass
  if (!gatewayToken) {
    return (
      <div>
        <h2>Step 1: Verify Your Identity</h2>
        <p>AGORA requires identity verification to ensure one person = one account.</p>
        <IdentityButton />
        {/* OR custom button: */}
        <button onClick={requestGatewayToken}>
          Verify with Civic Pass
        </button>
      </div>
    );
  }
  
  // User has valid pass, show registration
  return (
    <div>
      <h2>Step 2: Complete Registration</h2>
      <p>✅ Identity verified via Civic Pass</p>
      <BiometricCapture onCapture={handleBiometricCapture} />
      <button onClick={() => registerUser(gatewayToken)}>
        Claim Your AGORA
      </button>
    </div>
  );
}

// Registration transaction
async function registerUser(gatewayToken: GatewayToken) {
  const tx = await program.methods
    .registerUser(ageInDays, [], biometricHash)
    .accounts({
      userState: userStatePDA,
      protocolState: protocolStatePDA,
      biometricEntry: biometricEntryPDA,
      biometricRegistry: biometricRegistryPDA,
      mint: AGORA_MINT,
      userTokenAccount: userATA,
      gatewayToken: gatewayToken.publicKey,  // Civic Pass
      civicPass: gatewayToken.publicKey,
      user: wallet.publicKey,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .rpc();
}
```

#### Implementation Timeline

| Phase | Task | Duration | Status |
|-------|------|----------|--------|
| 1 | Add `solana-gateway` dependency | 1 hour | ⬜ Pending |
| 2 | Update RegisterUser accounts | 2 hours | ⬜ Pending |
| 3 | Implement verify_civic_pass_production() | 4 hours | ⬜ Pending |
| 4 | Add error codes | 1 hour | ⬜ Pending |
| 5 | Frontend GatewayProvider integration | 4 hours | ⬜ Pending |
| 6 | Devnet testing with test passes | 8 hours | ⬜ Pending |
| 7 | Integration tests | 8 hours | ⬜ Pending |
| **Total** | | **~3 days** | |

#### Security Considerations

**Privacy Model:**
- Personal data (name, ID number, photo) stored OFF-chain by Civic
- On-chain: only cryptographic attestation (Gateway Token)
- AGORA never sees or stores personal identity data
- Biometric hash is one-way (cannot reverse to fingerprint)

**Attack Vectors Mitigated:**

| Attack | Civic Pass | Biometric Hash | Combined |
|--------|------------|----------------|----------|
| Bot registration | ✅ Blocked | ✅ Blocked | ✅✅ |
| Fake identity | ✅ Blocked | ⚠️ Partial | ✅ |
| Multiple wallets (same person) | ⚠️ Partial | ✅ Blocked | ✅ |
| Stolen identity | ✅ Liveness check | ⚠️ If biometric stolen | ✅ |
| Dead person fraud | ✅ Liveness check | ✅ Annual liveness | ✅✅ |

### 4.2 Sybil Resistance

**Multi-layer Protection:**
1. **Civic Pass:** One person = one pass (government ID verified)
2. **Biometric Hash:** Fingerprint hash ensures uniqueness (PDA deduplication)
3. **Annual Liveness:** Proves user is currently alive (prevents dead person fraud)
4. **Behavioral Analysis:** Abnormal patterns flagged (future enhancement)

**Why Both Civic AND Biometric?**

Civic Pass alone has a weakness: a person could potentially create multiple wallets and verify each one separately. While Civic has internal deduplication, the biometric hash provides AGORA-specific, on-chain enforcement:

```
Scenario: Attacker has valid government ID, tries to create 2 accounts

Attempt 1:
- Wallet A + Civic Pass + Biometric Hash X → ✅ Success
- BiometricEntry PDA created with hash X

Attempt 2 (different wallet):
- Wallet B + New Civic Pass + Biometric Hash X → ❌ FAIL
- BiometricEntry PDA already exists for hash X
- Error: BiometricAlreadyRegistered
```

---

## 5. Distribution Mechanism

### 5.1 Claiming Process

```rust
pub fn calculate_claimable(user: &UserState) -> u64 {
    let current_time = Clock::get()?.unix_timestamp;
    let seconds_elapsed = current_time - user.last_claim_timestamp;
    let days_elapsed = (seconds_elapsed / 86400) as u64;
    
    // Cap at 30 days accumulation
    let claimable_days = min(days_elapsed, 30);
    
    claimable_days * DAILY_AMOUNT // 100 AGORA
}
```

### 5.2 Child Protection

```rust
pub fn handle_child_account(user: &UserState, current_time: i64) -> Result<()> {
    if user.is_child {
        let age_in_seconds = current_time - user.birth_timestamp;
        let age_in_years = age_in_seconds / (365 * 86400);
        
        match age_in_years {
            0..=12 => {
                // Fully locked, accumulates but cannot transfer
                user.transfer_locked = true;
            },
            13..=15 => {
                // 10% accessible for education/needs
                user.accessible_percentage = 10;
            },
            16..=17 => {
                // 25% accessible for independence prep
                user.accessible_percentage = 25;
            },
            _ => {
                // Full access at 18
                user.is_child = false;
                user.transfer_locked = false;
                user.accessible_percentage = 100;
            }
        }
    }
    Ok(())
}
```

---

## 6. Inflation Model

### 6.1 Mathematical Model

```
Annual Inflation Rate = (New Tokens Minted / Total Supply) Ã— 100

Where:
- New Tokens = (Active Users Ã— 100 Ã— 365) + One-time Retroactive Claims
- Total Supply = Previous Supply + New Tokens - Burned Tokens
```

### 6.2 Projected Inflation Curve

| Year | Inflation Rate | Total Supply | Users    | Key Factors                |
|------|---------------|--------------|----------|----------------------------|
| 1    | ~90%          | 26.2T        | 366M     | High retroactive claims    |
| 5    | 23.5%         | 91.2T        | 473M     | Growth phase              |
| 10   | 12.6%         | 194.8T       | 608M     | Stabilizing               |
| 15   | 8.6%          | 321.4T       | 728M     | Maturing                  |
| 20   | 6.4%          | 466.8T       | 822M     | Near equilibrium          |
| 30   | ~4%           | 820T         | 950M     | Steady state              |

### 6.3 Equilibrium Mechanics

```
At equilibrium:
Daily Mint Rate â‰ˆ Daily Burn Rate

Example (Year 20):
- Daily mint: 822M users Ã— 100 = 82.2B AGORA
- Daily transfer volume: ~1.6T AGORA
- Daily burn (0.025% avg): ~400M AGORA
- Treasury collection: ~400M AGORA
```

---

## 7. Governance Structure

### 7.1 Pure Direct Democracy

**NO GUARDIANS. NO MULTISIGS. COMMUNITY ONLY.**

```
Voting Power Distribution:
â”œâ”€â”€ Phase 1 (Year 0-1): Initial contributors (1 person = 1 vote)
â”œâ”€â”€ Phase 2 (Year 1+): All verified users (1 person = 1 vote)
â””â”€â”€ Not token-weighted (prevents plutocracy)
```

### 7.2 Proposal System

```rust
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub title: String,
    pub description: String,
    pub amount_requested: u64,
    pub recipient: Pubkey,
    pub vote_start: i64,
    pub vote_end: i64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub status: ProposalStatus,
    pub execution_timestamp: Option<i64>,
}

pub enum ProposalStatus {
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
}
```

### 7.3 Voting Parameters

| Tier | Treasury Amount | Duration | Quorum | Pass Threshold |
|------|----------------|----------|---------|---------------|
| 1    | <1 SOL         | 24 hours | 20%     | >50%          |
| 2    | 1-10 SOL       | 3 days   | 30%     | >50%          |
| 3    | >10 SOL        | 7 days   | 40%     | >66%          |
| Constitutional | Any    | 14 days  | 50%     | >75%          |

---

## 8. Gas Pool Economics & Security

**STATUS: ✅ IMPLEMENTED in lib.rs v3.4**

### 8.1 Problem Statement

Every Solana transaction requires SOL for gas fees (~0.00025 SOL). This creates a barrier for new users, especially in developing nations. Without a solution, AGORA cannot achieve true universal access.

### 8.2 Implementation Status

| Component | Status | Location in lib.rs |
|-----------|--------|-------------------|
| Gas Pool Constants | ✅ Done | Lines 161-206 |
| GasPoolState struct | ✅ Done | After ProtocolState |
| SponsorRecord struct | ✅ Done | After GasPoolState |
| TransactionTracker | ✅ Done | After SponsorRecord |
| initialize_gas_pool() | ✅ Done | Gas Pool Management section |
| sponsor_gas_pool() | ✅ Done | Gas Pool Management section |
| claim_gas_subsidy() | ✅ Done | Gas Pool Management section |
| record_transaction() | ✅ Done | Anti-abuse detection |
| emergency_pause/resume | ✅ Done | Emergency controls |
| Helper functions | ✅ Done | Before handle_fees() |
| Validation contexts | ✅ Done | After Liveness contexts |
| Events | ✅ Done | After BusinessRegistrationFrozen |
| Error codes | ✅ Done | End of file |

### 8.3 SOL Gas Pool System

#### Pool Structure (IMPLEMENTED in lib.rs)
```rust
/// Gas Pool state - manages SOL subsidies for transaction fees.
/// PDA seeded by ["gas_pool_state"]
#[account]
pub struct GasPoolState {
    pub authority: Pubkey,              // PDA authority
    pub total_deposited: u64,           // Total SOL contributed (lifetime)
    pub total_distributed: u64,         // SOL used for gas subsidies
    pub available_balance: u64,         // Current available SOL
    pub daily_distributed: u64,         // Amount in current 24h period
    pub daily_reset_time: i64,          // When daily counter resets
    pub total_sponsors: u64,            // Number of sponsors
    pub emergency_paused: bool,         // Emergency mode flag
    pub emergency_triggered_at: i64,    // When emergency was triggered
    pub emergency_reason: EmergencyReason,
    pub bump: u8,
}

/// Individual sponsor record - PDA seeded by ["sponsor", user_pubkey]
#[account]
pub struct SponsorRecord {
    pub sponsor: Pubkey,
    pub tier: SponsorTier,
    pub total_contributed: u64,
    pub personal_allocation: u64,       // Remaining personal gas
    pub pool_contribution: u64,         // Amount to community pool
    pub sponsored_at: i64,
    pub tier_upgraded_at: i64,
    pub daily_subsidy_count: u8,
    pub daily_reset_time: i64,
    pub bump: u8,
}

/// Anti-abuse transaction tracker - PDA seeded by ["tx_tracker", user_pubkey]
#[account]
pub struct TransactionTracker {
    pub user: Pubkey,
    pub recent_recipients: [Pubkey; 10],  // Circular buffer
    pub recent_timestamps: [i64; 10],
    pub buffer_index: u8,
    pub tx_last_minute: u8,
    pub minute_start: i64,
    pub suspicion_score: u8,              // 0-100
    pub bump: u8,
}
```

**Key Properties:**
- 100% trustless - PDA controlled
- No human gatekeepers
- Transparent on-chain
- Separate from DAO treasury

### 8.4 Sponsor Tier System (IMPLEMENTED)

#### Constants in lib.rs
```rust
// Sponsor tier thresholds (in lamports, 1 SOL = 1_000_000_000)
pub const BRONZE_THRESHOLD: u64 = 1_000_000_000;           // 1 SOL
pub const SILVER_THRESHOLD: u64 = 10_000_000_000;          // 10 SOL
pub const GOLD_THRESHOLD: u64 = 100_000_000_000;           // 100 SOL
pub const PLATINUM_THRESHOLD: u64 = 1_000_000_000_000;     // 1,000 SOL
pub const DIAMOND_THRESHOLD: u64 = 10_000_000_000_000;     // 10,000 SOL

// Personal allocation percentages
pub const BRONZE_PERSONAL_PCT: u64 = 20;
pub const SILVER_PERSONAL_PCT: u64 = 15;
pub const GOLD_PERSONAL_PCT: u64 = 10;
pub const PLATINUM_PERSONAL_PCT: u64 = 5;
pub const DIAMOND_PERSONAL_PCT: u64 = 3;
```

#### Tiered Personal Allocation Model

| Tier | Contribution | Personal Use | To Pool | TX Budget | Fee Discount |
|------|-------------|--------------|---------|-----------|--------------|
| ðŸ¥‰ Bronze | 1 SOL | 20% (0.2 SOL) | 80% | 800 TX | 25% |
| ðŸ¥ˆ Silver | 10 SOL | 15% (1.5 SOL) | 85% | 6,000 TX | 50% |
| ðŸ¥‡ Gold | 100 SOL | 10% (10 SOL) | 90% | 40,000 TX | 75% |
| ðŸ† Platinum | 1,000 SOL | 5% (50 SOL) | 95% | 200,000 TX | 100% |
| ðŸ’Ž Diamond | 10,000 SOL | 3% (300 SOL) | 97% | 1.2M TX | 100% + perks |

**Why Tiered Percentages?**
- Small sponsors need higher % for daily use
- Large sponsors are likely institutions/merchants
- Prevents any single actor from draining pool
- Creates sustainable economics

#### Implementation (Actual Code from lib.rs)
```rust
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

pub fn get_sponsor_fee_discount(tier: &SponsorTier) -> u64 {
    match tier {
        SponsorTier::None => 0,
        SponsorTier::Bronze => BRONZE_FEE_DISCOUNT,
        SponsorTier::Silver => SILVER_FEE_DISCOUNT,
        SponsorTier::Gold => GOLD_FEE_DISCOUNT,
        SponsorTier::Platinum => PLATINUM_FEE_DISCOUNT,
        SponsorTier::Diamond => DIAMOND_FEE_DISCOUNT,
    }
}
```

### 8.5 User Access Tiers (IMPLEMENTED)

```rust
// Constants for free tier limits
pub const FREE_TIER_DAILY_TX: u8 = 5;
pub const FREE_TIER_COOLDOWN: i64 = 60;  // 60 seconds between TX
pub const MIN_SUBSIDIZED_AMOUNT: u64 = 100_000_000_000;  // 100 AGORA minimum

// Access determined by sponsor_tier in UserState:
// - NonContributor: Monthly claims, 5 TX/day, 60s cooldown
// - Sponsor: Daily claims, uses personal allocation, no cooldown
// - Depleted Sponsor: Falls back to free tier limits
```

**Access Rules:**
1. **Non-contributors:** Monthly claims, 5 TX/day limit
2. **Sponsors:** Daily claims, use personal allocation
3. **When personal depleted:** Drop to free tier

### 8.6 Attack Vectors & Mitigations (IMPLEMENTED)

#### 8.6.1 Ping-Pong Attack
**Attack:** Two accounts rapidly exchange tokens to drain gas
```
A → B → A → B (repeat thousands of times)
```

**Mitigation (Actual Implementation):**
```rust
pub const PING_PONG_WINDOW: i64 = 300;  // 5 minutes

fn detect_ping_pong(
    tracker: &TransactionTracker,
    sender: &Pubkey,
    recipient: &Pubkey,
    current_time: i64,
) -> bool {
    for i in 0..10 {
        let prev_recipient = tracker.recent_recipients[i];
        let prev_time = tracker.recent_timestamps[i];
        
        if prev_time == 0 { continue; }
        if current_time - prev_time > PING_PONG_WINDOW { continue; }
        
        // If we previously sent to this person, and now they're the sender
        if prev_recipient == *sender {
            return true;  // PING-PONG DETECTED!
        }
    }
    false
}
```

#### 8.6.2 Circular Trading Attack
**Attack:** A→B→C→D→A circular flow to drain pool

**Mitigation (Actual Implementation):**
```rust
pub const MAX_SAME_RECIPIENT_PER_HOUR: u8 = 5;

fn count_same_recipient(
    tracker: &TransactionTracker,
    recipient: &Pubkey,
    current_time: i64,
    window_seconds: i64,
) -> u8 {
    let mut count = 0u8;
    for i in 0..10 {
        if tracker.recent_recipients[i] == *recipient {
            if current_time - tracker.recent_timestamps[i] <= window_seconds {
                count += 1;
            }
        }
    }
    count
}
```

#### 8.6.3 Dust Attack
**Attack:** Thousands of micro-transactions (<10 AGORA) to drain gas
```
Send 1 AGORA × 10,000 times = 2.5 SOL drained
```

**Mitigation (Actual Implementation):**
```rust
pub const MIN_SUBSIDIZED_AMOUNT: u64 = 100_000_000_000; // 100 AGORA

fn apply_free_tier_limits(
    daily_count: u8,
    tracker: &TransactionTracker,
    current_time: i64,
) -> Result<u64> {
    // Check daily limit
    require!(daily_count < FREE_TIER_DAILY_TX, ErrorCode::DailySubsidyLimitExceeded);
    
    // Check cooldown (60 seconds)
    if tracker.buffer_index > 0 {
        let last_tx_time = tracker.recent_timestamps[(tracker.buffer_index - 1) as usize];
        require!(
            current_time - last_tx_time >= FREE_TIER_COOLDOWN,
            ErrorCode::SubsidyCooldownActive
        );
    }
    
    Ok(estimate_gas_cost(&TransactionType::Transfer))
}
```

#### 8.6.4 Sybil Attack
**Attack:** Create 1000 fake accounts to claim gas subsidies

**Mitigation:**
- Civic Pass requirement (1 person = 1 account)
- Biometric deduplication (fingerprint hash)
- Behavioral analysis via TransactionTracker
- Progressive verification levels

### 8.7 Rate Limiting System (IMPLEMENTED)

```rust
pub const RAPID_TX_THRESHOLD: u8 = 10;  // >10 TX in 1 minute = suspicious

// In record_transaction():
// Track TX count per minute
if current_time - tracker.minute_start > 60 {
    tracker.tx_last_minute = 1;
    tracker.minute_start = current_time;
} else {
    tracker.tx_last_minute += 1;
}

// Check for rapid fire
if tracker.tx_last_minute > RAPID_TX_THRESHOLD {
    tracker.suspicion_score += 30;  // Significant penalty
}

impl RateLimiter {
    fn check_limits(&self, user: &User, tx: &Transaction) -> Result<()> {
        // Check time since last TX
        if tx.timestamp - user.last_tx < self.min_tx_interval {
            return Err(ErrorCode::TooSoon);
        }
        
        // Check daily limits
        if user.tx_count_today >= self.daily_limit {
            return Err(ErrorCode::DailyLimitExceeded);
        }
        
        // Check subsidy eligibility
        if user.subsidized_today >= self.subsidized_daily_limit {
            tx.subsidy_eligible = false;  // Pay your own gas
        }
        
        Ok(())
    }
}
```

### 8.7 Penalty System

**Zero Tolerance Policy:**
```rust
pub enum Violation {
    FirstViolation => Suspension(Days(30)),
    SecondViolation => PermanentBlacklist,
}
```

**No appeals. No exceptions. No mercy.**

**What triggers penalties:**
- Ping-pong patterns
- Circular trading
- Dust attacks
- Rate limit violations
- Anomalous behavior

### 8.8 Penalty System (IMPLEMENTED)

**Zero Tolerance Policy:**
```rust
pub const SUSPENSION_DAYS: i64 = 30;

// In record_transaction(), after detecting violations:
if is_ping_pong || same_recipient_count > MAX_SAME_RECIPIENT_PER_HOUR {
    user.violations += 1;
    
    if user.violations >= 2 {
        user.permanently_blacklisted = true;
        emit!(IndividualBanned { ... });
    } else {
        user.suspended_until = current_time + (SUSPENSION_DAYS * SECONDS_PER_DAY);
        emit!(IndividualFraudDetected { ... });
    }
}
```

**No appeals. No exceptions. No mercy.**

### 8.9 Emergency Brake System (IMPLEMENTED)

```rust
pub const DAILY_DRAIN_WARNING: u64 = 100_000_000_000;     // 100 SOL/day
pub const DAILY_DRAIN_EMERGENCY: u64 = 500_000_000_000;   // 500 SOL/day
pub const MIN_POOL_BALANCE: u64 = 10_000_000_000;         // 10 SOL minimum

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, PartialEq)]
pub enum EmergencyReason {
    #[default]
    None,
    ExcessiveDrain,
    AttackDetected,
    LowBalance,
    ManualPause,
}

// Auto-triggered in claim_gas_subsidy():
if gas_pool.daily_distributed >= DAILY_DRAIN_EMERGENCY {
    gas_pool.emergency_paused = true;
    gas_pool.emergency_reason = EmergencyReason::ExcessiveDrain;
    emit!(GasPoolEmergency { ... });
    return Err(ErrorCode::GasPoolEmergencyPause.into());
}

if gas_pool.available_balance < MIN_POOL_BALANCE {
    gas_pool.emergency_paused = true;
    gas_pool.emergency_reason = EmergencyReason::LowBalance;
    return Err(ErrorCode::GasPoolLowBalance.into());
}
```

### 8.10 Core Functions (IMPLEMENTED)

| Function | Purpose | Key Logic |
|----------|---------|-----------|
| `initialize_gas_pool()` | Create pool PDA | Sets up GasPoolState with default values |
| `sponsor_gas_pool(amount)` | Accept SOL deposits | Calculates tier, splits personal/pool, transfers SOL |
| `claim_gas_subsidy(tx_type)` | Distribute gas | Checks limits, uses personal or pool, transfers SOL |
| `record_transaction(recipient)` | Anti-abuse tracking | Detects ping-pong, same-recipient, rapid fire |
| `emergency_pause_gas_pool(reason)` | Emergency stop | Sets paused flag, emits alert |
| `resume_gas_pool()` | Resume operations | Clears paused flag |

### 8.11 Economic Sustainability

#### Pool Longevity Calculation
```
Initial pool: 10,000 SOL
Users: 10,000 monthly claimers
Cost per user: 0.003 SOL/month
Monthly burn: 30 SOL
Runway: 333 months (27+ years)
```

#### Growth Projections
```
Year 1: 100 sponsors × avg 100 SOL = 10,000 SOL
Year 2: 1,000 sponsors × avg 50 SOL = 50,000 SOL
Year 3: 10,000 sponsors × avg 20 SOL = 200,000 SOL
Total: 260,000 SOL = Covers 1M users for 20+ years
```

### 8.12 Implementation Status

| Component | Status | Location in lib.rs |
|-----------|--------|-------------------|
| Gas Pool Constants | ✅ DONE | Lines 161-206 |
| GasPoolState struct | ✅ DONE | After ProtocolState |
| SponsorRecord struct | ✅ DONE | After GasPoolState |
| TransactionTracker | ✅ DONE | After SponsorRecord |
| EmergencyReason enum | ✅ DONE | After TransactionTracker |
| initialize_gas_pool() | ✅ DONE | Gas Pool Management section |
| sponsor_gas_pool() | ✅ DONE | Gas Pool Management section |
| claim_gas_subsidy() | ✅ DONE | Gas Pool Management section |
| record_transaction() | ✅ DONE | Anti-abuse detection |
| emergency_pause/resume | ✅ DONE | Emergency controls |
| Helper functions (7) | ✅ DONE | Before handle_fees() |
| Validation contexts (5) | ✅ DONE | After Liveness contexts |
| Events (6 new) | ✅ DONE | Events section |
| Error codes (6 new) | ✅ DONE | Error codes section |

---

## 9. Merchant Economy & Auto-Detection

### 9.1 Merchant Problem Statement

Merchants are critical for AGORA adoption but face challenges:
- Setup complexity and paperwork
- Transaction fees eating into margins  
- Gas fee management
- Integration difficulties
- Cash flow management with crypto volatility

**Solution:** Automatic merchant detection with progressive benefits - no applications, no waiting, just rewards for being a merchant.

### 9.2 Automatic Merchant Detection Algorithm

#### 30-Day Rolling Window

Merchant status is based on **30-day rolling metrics**, not lifetime totals. This ensures:

- Active merchants maintain their status through ongoing activity
- Inactive merchants naturally decay (see decay rules below)
- Fair measurement across all merchants regardless of account age

```rust
// Monthly tracking fields in UserState
pub monthly_volume_received: u128,     // Volume in current 30-day period
pub monthly_volume_period_start: i64,  // When current period started
pub monthly_unique_payers: Vec<Pubkey>, // Unique payers in current period
pub monthly_unique_count: u32,         // Count for current period

// Reset happens automatically when period expires
fn maybe_reset_monthly_period(user: &mut UserState, current_time: i64) -> bool {
    if current_time - user.monthly_volume_period_start >= 30 * SECONDS_PER_DAY {
        user.monthly_volume_received = 0;
        user.monthly_volume_period_start = current_time;
        user.monthly_unique_payers.clear();
        user.monthly_unique_count = 0;
        return true;  // Period was reset
    }
    false
}
```

#### Core Detection Logic - Simple OR

Merchants qualify via **EITHER** unique customers **OR** monthly volume. This ensures both types of merchants can qualify:

- **Retail merchants** (kavarna, trgovina): Many customers, smaller transactions
- **Service merchants** (vodovodar, odvetnik): Few customers, large transactions

```rust
/// Determines merchant tier using simple OR logic
fn determine_merchant_tier(unique_customers: u32, monthly_volume: u128) -> MerchantTier {
    // Enterprise: ≥2,000 customers OR ≥500,000 AGORA
    if unique_customers >= 2000 || monthly_volume >= 500_000 {
        return MerchantTier::Enterprise;
    }
    
    // Large: ≥500 customers OR ≥100,000 AGORA
    if unique_customers >= 500 || monthly_volume >= 100_000 {
        return MerchantTier::Large;
    }
    
    // Medium: ≥100 customers OR ≥25,000 AGORA
    if unique_customers >= 100 || monthly_volume >= 25_000 {
        return MerchantTier::Medium;
    }
    
    // Small: ≥25 customers OR ≥5,000 AGORA
    if unique_customers >= 25 || monthly_volume >= 5_000 {
        return MerchantTier::Small;
    }
    
    // Emerging: ≥10 customers OR ≥1,000 AGORA
    if unique_customers >= 10 || monthly_volume >= 1_000 {
        return MerchantTier::Emerging;
    }
    
    MerchantTier::None
}
```

#### Detection Thresholds

| Tier | Unique Customers | OR | Monthly Volume |
|------|------------------|-----|----------------|
| Emerging | ≥10 | OR | ≥1,000 AGORA |
| Small | ≥25 | OR | ≥5,000 AGORA |
| Medium | ≥100 | OR | ≥25,000 AGORA |
| Large | ≥500 | OR | ≥100,000 AGORA |
| Enterprise | ≥2,000 | OR | ≥500,000 AGORA |

#### Real-World Examples

| Person | Unique Customers | Monthly Volume | Tier | Why |
|--------|------------------|----------------|------|-----|
| Kavarna | 200 | 8,000 AGORA | Medium | 200 ≥ 100 customers |
| Vodovodar | 3 | 30,000 AGORA | Medium | 30k ≥ 25k volume |
| Frizer | 50 | 15,000 AGORA | Small | 50 ≥ 25 customers |
| Odvetnik | 1 | 80,000 AGORA | Medium | 80k ≥ 25k volume |
| Velika trgovina | 2,500 | 400,000 AGORA | Enterprise | 2500 ≥ 2000 customers |

### 9.3 Merchant Benefits Package

#### Economic Benefits
```rust
pub struct MerchantEconomicBenefits {
    // Fee structure
    outgoing_fee_discount: u8,      // 25-100% based on tier
    bulk_payment_discount: u8,      // Extra 5-25% on batch
    
    // Special rates
    employee_payment_rate: u8,      // 0-50% based on tier
    supplier_payment_rate: u8,      // Reduced rate
    
    // Cashback program
    customer_rewards_pool: bool,    // Can offer cashback
    loyalty_points_system: bool,    // Issue merchant tokens
}
```

#### Benefit Tiers

| Merchant Tier | Outgoing Fee | Batch Discount | Employee Payments | API Calls/Month |
|--------------|--------------|----------------|-------------------|-----------------|
| Emerging | 25% off | 5% extra | 50% off | 1,000 |
| Small | 50% off | 10% extra | Free | 10,000 |
| Medium | 75% off | 15% extra | Free | 100,000 |
| Large | 100% off | 20% extra | Free + subsidy | 1,000,000 |
| Enterprise | 100% off | 25% extra | Free + subsidy + priority | Unlimited |

#### Operational Benefits
```rust
pub struct MerchantOperationalBenefits {
    // Payment features
    batch_payments: bool,           // Pay multiple recipients in 1 TX
    recurring_payments: bool,       // Automated subscriptions
    instant_settlement: bool,       // No waiting period
    payment_scheduling: bool,       // Future-dated payments
    
    // Integration tools
    pos_terminal_app: bool,         // Free POS software
    qr_code_generator: bool,        // Dynamic QR codes
    payment_links: bool,            // Shareable payment URLs
    invoice_system: bool,           // Built-in invoicing
    
    // Analytics
    dashboard_access: bool,         // Merchant dashboard
    sales_analytics: bool,         // Revenue tracking
    customer_insights: bool,        // Buyer behavior data
    export_capabilities: bool,      // CSV/JSON/XML export
    
    // Support
    priority_support: bool,         // 24/7 helpline
    dedicated_manager: bool,        // Personal contact
    onboarding_assistance: bool,    // Setup help
    custom_integration: bool,       // API customization
}
```

### 9.4 Employee & Payroll System

#### Automatic Employee Detection
```rust
pub fn detect_employee_relationship(
    merchant: &UserState,
    recipient: &UserState,
) -> bool {
    // Regular payments (weekly/biweekly/monthly)
    let payment_pattern = analyze_payment_frequency(
        &merchant.payments_to[recipient.pubkey]
    );
    
    // Consistent amounts (salary)
    let amount_consistency = calculate_amount_variance(
        &merchant.payments_to[recipient.pubkey]
    );
    
    payment_pattern.is_regular() && amount_consistency < 0.1
}
```

#### Payroll Benefits
- **Zero fees** on all employee payments
- **Batch processing** - pay all employees in one transaction
- **Gas subsidies** for employees from merchant's pool
- **Automated scheduling** - set and forget monthly payments
- **Tax reporting** exports for accounting

### 9.5 DAO Treasury Incentive System

#### Why Contribute to DAO Treasury?

Current problem: Gas pool has clear benefits, DAO treasury has none.

#### DAO Contributor Benefits
```rust
pub struct DaoContributorBenefits {
    // Governance power
    voting_weight_multiplier: f64,    // 1.0Ã— to 2.0Ã— based on contribution
    proposal_rights: ProposalTier,    // Can propose without waiting
    veto_participation: bool,          // Can participate in emergency votes
    
    // Economic benefits
    fee_discount: u8,                 // Up to 50% (not 100% - reserved for gas pool)
    priority_tx_processing: bool,      // Skip mempool queues
    
    // Access benefits  
    early_feature_access: bool,       // Beta features first
    governance_dashboard: bool,        // Advanced analytics
    
    // Recognition
    contributor_badge: DaoTier,        // On-chain recognition
    public_acknowledgment: bool,       // Listed in UI/docs
}
```

#### DAO Contribution Tiers

| Tier | Contribution | Voting Weight | Fee Discount | Special Rights |
|------|-------------|---------------|--------------|----------------|
| Supporter | 0.1+ SOL | 1.0Ã— | 10% | Basic voting |
| Contributor | 1+ SOL | 1.1Ã— | 20% | Proposal comments |
| Builder | 10+ SOL | 1.25Ã— | 30% | Create proposals |
| Sustainer | 100+ SOL | 1.5Ã— | 40% | Priority queue |
| Founder | 1000+ SOL | 2.0Ã— | 50% | Emergency votes |

### 9.6 Merchant Onboarding Flow

#### Automatic Progressive Onboarding
```
Day 1: Regular user
â†“ (Receives more than sends)
Day 7: Pattern detected â†’ "Emerging Merchant" status
â†“ (Reaches 10Ã— ratio, 50 customers)
Day 30: "Small Merchant" â†’ Benefits activated automatically
â†“ (Growth continues)
Day 90: "Medium Merchant" â†’ More benefits unlocked
â†“ (Sustained growth)
Year 1: "Large Merchant" â†’ Full benefits package
```

#### Notification System
```rust
pub fn check_merchant_progression(user: &UserState) -> Option<Notification> {
    let current = user.merchant_status;
    let projected = calculate_merchant_status(user);
    
    if projected > current {
        Some(Notification {
            title: "Congratulations! Merchant Status Upgraded",
            message: format!(
                "You're now a {} Merchant! New benefits unlocked:",
                projected
            ),
            benefits: list_new_benefits(current, projected),
            action: "View Dashboard",
        })
    } else if close_to_next_tier(user) {
        Some(Notification {
            title: "You're close to the next merchant tier!",
            message: format!(
                "Only {} more customers to become {} Merchant",
                customers_needed(user),
                next_tier(current)
            ),
        })
    } else {
        None
    }
}
```

### 9.7 Integration Tools

#### REST API for Merchants
```javascript
// Payment processing
POST /api/v1/merchant/payment
{
  "recipient": "wallet_address",
  "amount": 100.50,
  "currency": "AGORA",
  "reference": "INV-2025-001",
  "webhook": "https://merchant.com/callback"
}

// Batch payments
POST /api/v1/merchant/batch
{
  "payments": [
    {"to": "employee1", "amount": 5000},
    {"to": "employee2", "amount": 4500},
    {"to": "supplier1", "amount": 10000}
  ]
}

// Analytics
GET /api/v1/merchant/analytics
{
  "period": "monthly",
  "metrics": ["revenue", "customers", "average_transaction"]
}
```

#### SDK Libraries
- JavaScript/TypeScript
- Python
- PHP (for WooCommerce)
- Java (for enterprise)
- Mobile SDKs (iOS/Android)

### 9.8 Economic Impact Modeling

#### Merchant Adoption Curve
```
Month 1: 10 merchants (early adopters)
Month 3: 100 merchants (word of mouth)
Month 6: 1,000 merchants (network effects)
Year 1: 10,000 merchants (critical mass)
Year 2: 100,000 merchants (mainstream)
```

#### Transaction Volume Projection
```
Regular users: 2-3 TX/day average
Merchants: 50-200 TX/day average
Impact: 1000 merchants = 100,000 regular users in TX volume
```

#### Fee Revenue vs Benefits Cost
```rust
fn calculate_merchant_economics() -> Economics {
    // Merchants pay less fees but generate more volume
    let lost_fee_revenue = merchant_count * average_discount * base_fee;
    let gained_volume_revenue = merchant_tx_volume * regular_fee;
    let network_growth_value = network_effect_multiplier * merchant_count;
    
    Economics {
        direct_loss: lost_fee_revenue,
        direct_gain: gained_volume_revenue,
        network_value: network_growth_value,
        net_benefit: gained_volume_revenue + network_growth_value - lost_fee_revenue,
    }
}
```

### 9.9 Implementation Priority

**Phase 1 (Launch):** Basic detection algorithm
**Phase 2 (Month 1):** Automatic tier assignment
**Phase 3 (Month 2):** Benefit distribution system
**Phase 4 (Month 3):** API & integration tools
**Phase 5 (Month 6):** Full merchant dashboard

---

## 10. DAO Proposal System (IMPLEMENTED)

### 10.1 Overview

The DAO governance system enables democratic decision-making with strong anti-spam protections. Every verified human has exactly one vote, regardless of token holdings.

### 10.2 Proposal Types

```rust
pub enum ProposalType {
    Standard,       // General governance decisions
    Treasury,       // Spending treasury funds
    Constitutional, // Protocol parameter changes
    Sanction,       // Country sanctions
}
```

### 10.3 Proposal Parameters

| Type | Bond | Quorum | Approval | Voting Period |
|------|------|--------|----------|---------------|
| Standard | 20,000 AGORA | 1,000 votes | >50% | 3 days |
| Treasury | 50,000 AGORA | 5,000 votes | >50% | 7 days |
| Constitutional | 75,000 AGORA | 10,000 votes | >67% | 14 days |
| Sanction | 100,000 AGORA | 25,000 votes | >75% | 14 days |

### 10.4 Bond Mechanism

Bonds prevent spam attacks by requiring significant stake:

```rust
// Bond amounts in AGORA base units (9 decimals)
pub const PROPOSAL_BOND_STANDARD: u64 = 20_000_000_000_000;      // 20,000 AGORA
pub const PROPOSAL_BOND_TREASURY: u64 = 50_000_000_000_000;      // 50,000 AGORA
pub const PROPOSAL_BOND_CONSTITUTIONAL: u64 = 75_000_000_000_000; // 75,000 AGORA
pub const PROPOSAL_BOND_SANCTION: u64 = 100_000_000_000_000;     // 100,000 AGORA
```

**Bond Return Rules:**
- Proposal reaches 50%+ of quorum → Bond returned (even if rejected)
- Proposal fails to reach 50% of quorum → Bond forfeited to treasury

### 10.5 Reputation System

Reputation is calculated automatically from voting results (no manual flagging):

```rust
// Reputation changes based on voting outcome
pub const REP_PROPOSAL_PASSED: i32 = 2;    // Passed (>50% yes + quorum)
pub const REP_PROPOSAL_REJECTED: i32 = 1;  // Rejected but reached quorum
pub const REP_NO_QUORUM_50: i32 = -1;      // 50%+ of quorum reached
pub const REP_NO_QUORUM_25: i32 = -2;      // 25-50% of quorum
pub const REP_NO_QUORUM_10: i32 = -3;      // <25% of quorum (obvious spam)

// Ban threshold
pub const REP_THRESHOLD_BAN: i32 = -10;    // Cannot create proposals
```

**Bond Multiplier Formula:**
```rust
// bond_multiplier = 1 + (abs(reputation) / 2)
let bond_multiplier: u64 = if proposer_state.proposal_reputation >= 0 {
    1
} else {
    1 + (proposer_state.proposal_reputation.abs() / 2) as u64
};
```

| Reputation | Multiplier | Effect |
|------------|------------|--------|
| 0+ | 1× | Normal bond |
| -2 to -3 | 2× | Double bond |
| -4 to -5 | 3× | Triple bond |
| -6 to -7 | 4× | Quadruple bond |
| -8 to -9 | 5× | 5× bond |
| -10 | ∞ | **Banned** |

### 10.6 Core Structs

```rust
#[account]
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub status: ProposalStatus,
    pub title: [u8; 64],
    pub description_hash: [u8; 32],  // IPFS hash
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
    pub treasury_amount: u64,        // For treasury proposals
    pub treasury_recipient: Pubkey,
    pub sanction_country: [u8; 3],   // ISO 3166-1 alpha-3
    pub bump: u8,
}

#[account]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub proposal_id: u64,
    pub choice: u8,  // 0=no, 1=yes, 2=abstain
    pub voted_at: i64,
    pub bump: u8,
}

#[account]
pub struct ProposalRegistry {
    pub next_proposal_id: u64,
    pub total_proposals: u64,
    pub active_proposals: u32,
    pub total_bonds_forfeited: u64,
    pub total_bonds_returned: u64,
    pub bump: u8,
}
```

### 10.7 Core Functions

```rust
// Create a new proposal
pub fn create_proposal(
    ctx: Context<CreateProposal>,
    proposal_type: ProposalType,
    title: [u8; 64],
    description_hash: [u8; 32],
    treasury_amount: u64,
    treasury_recipient: Pubkey,
    sanction_country: [u8; 3],
) -> Result<()>

// Vote on active proposal
pub fn vote_on_proposal(
    ctx: Context<VoteOnProposal>,
    choice: u8,  // 0=no, 1=yes, 2=abstain
) -> Result<()>

// Finalize proposal after voting ends
pub fn finalize_proposal(
    ctx: Context<FinalizeProposal>
) -> Result<()>
```

### 10.8 Security: Why No Manual Flagging

Manual flagging creates attack vectors:
- **Coordinated flag attacks** — 100 malicious users can flag any legitimate proposal
- **Unknowing users** — may click flag thinking it's "like"
- **Sybil vulnerability** — even with 1-person-1-vote, coordination is possible

**Solution:** Reputation calculated purely from voting results. No user input = no attack vector.

---

## 11. Country Sanctions (IMPLEMENTED)

### 11.1 Philosophy

AGORA includes a mechanism for global democratic accountability: country sanctions for human rights violations.

**Principles:**
- Punishes governments, not individuals (citizens still receive reduced UBI)
- Requires global supermajority (75% approval)
- Temporary with clear expiration
- Can be lifted early by DAO vote
- Alternative to military intervention

### 11.2 Sanction Parameters

| Parameter | Value |
|-----------|-------|
| Bond Required | 100,000 AGORA |
| Quorum | 25,000 votes |
| Approval Threshold | 75% supermajority |
| Voting Period | 14 days |
| Default Duration | 1 year |

### 11.3 Implementation

```rust
#[account]
pub struct CountrySanction {
    pub country_code: [u8; 3],      // ISO 3166-1 alpha-3
    pub reason: [u8; 256],          // Description
    pub ubi_percentage: u8,         // 1-99 (percentage of normal UBI)
    pub imposed_at: i64,
    pub expires_at: i64,
    pub proposal_id: u64,           // Originating proposal
    pub is_active: bool,
    pub lifted_early: bool,
    pub lifted_at: i64,
    pub bump: u8,
}
```

### 11.4 UBI Reduction (Never Zero)

Sanctions reduce UBI but never eliminate it:

```rust
// Calculate sanctioned UBI amount
let sanction_percentage = get_country_sanction_percentage(user.citizenship);
let daily_amount = if sanction_percentage < 100 {
    DAILY_UBI_AMOUNT * sanction_percentage as u64 / 100
} else {
    DAILY_UBI_AMOUNT  // No sanction
};
```

**Example:** If country X is sanctioned at 25% UBI, citizens receive 25 AGORA/day instead of 100.

### 11.5 Safeguards

- **Very high bond** (100,000 AGORA) prevents frivolous sanctions
- **75% supermajority** ensures broad global consensus
- **Never zero UBI** — we don't abandon victims of their government
- **Clear expiration** — sanctions don't last forever
- **Early lifting** — can be removed if situation improves
- **Full transparency** — all votes and sanctions on-chain

---

## 12. Security Considerations

### 12.1 Attack Vectors & Mitigations

| Attack Vector | Description | Mitigation |
|--------------|------------|------------|
| Sybil Attack | Creating multiple fake accounts | Civic Pass verification + one human = one account |
| Business Spam Attack | Creating thousands of fake businesses | Max 1 business per human account |
| Storage Bloat | Overloading chain with business data | Business limit + size constraints |
| Double Claiming | Claiming UBI multiple times | On-chain timestamp tracking |
| Governance Capture | Taking control of voting | One person one vote + quorum requirements |
| Smart Contract Bugs | Exploiting code vulnerabilities | Formal verification + audits |
| Oracle Manipulation | Attacking Civic Pass data | Multiple data sources for Civic |
| Front-running | Transaction order manipulation | Solana's parallel execution model |
| Infinite Mint Bug | Unlimited token creation | Hard-coded daily limits per user |
| Ping-pong Trading | Aâ†’Bâ†’A rapid trades for gas | 30-day suspension â†’ permanent ban |
| Circular Trading | Aâ†’Bâ†’Câ†’A wash trading | Network analysis + unique customer requirements |
| Merchant Status Gaming | Faking merchant metrics | Volume-based detection + time requirements |

### 10.2 Business Spam Attack Prevention (Critical)

#### The Attack
Without limits, a malicious actor could:
```rust
// ATTACK SCENARIO: Create unlimited businesses
for i in 0..10000 {
    register_business(format!("FakeBusiness_{}", i));
}

// Consequences:
// - Blockchain storage bloat (each business = ~500 bytes)
// - Performance degradation (iteration over businesses)
// - Confusion in marketplace (finding real businesses)
// - Potential DoS on indexers and explorers
```

#### The Defense
```rust
pub fn register_business() -> Result<()> {
    // CRITICAL: Hard limit of 1 business per human
    require!(
        user.business_profile.is_none(),
        ErrorCode::BusinessAlreadyExists
    );
    
    // Additional protections:
    // - Each business tied to verified human (Civic Pass)
    // - Businesses cannot claim UBI (only humans)
    // - Businesses cannot create other businesses
    // - All transactions traced to human owner
    // - Sub-divisions handled off-chain (IPFS/Arweave)
}
```

#### Why 1 is the Right Limit
- **Real-world accuracy:**
  - Most entities have divisions under ONE legal business
  - Example: "John's Services" with plumbing/electrical divisions
  - Matches actual business structure
  
- **Maximum security:**
  - Cannot create multiple spam businesses per account
  - Storage costs minimal (500 bytes per user)
  - Performance optimal (no iteration)
  
- **Flexibility preserved:**
  - Unlimited divisions via off-chain metadata
  - IPFS/Arweave for branch/location data
  - Employee management still possible

#### Off-Chain Metadata Structure
```json
{
  "main_business": "John's Services",
  "divisions": [
    {"name": "John's Plumbing", "type": "plumbing"},
    {"name": "John's Electric", "type": "electrical"}
  ],
  "locations": [
    {"address": "123 Main St", "type": "headquarters"},
    {"address": "456 Oak Ave", "type": "branch"}
  ]
}
```

#### Future Flexibility
```rust
// DAO can enable multi-business for specific users if needed
pub struct ProtocolState {
    pub single_business_enforced: bool,  // Currently true
}

// Special cases (large franchises) can petition DAO
pub fn dao_grant_multi_business(user: Pubkey) -> Result<()> {
    // Requires governance vote for exception
    // But default remains 1 for security
}
```

### 10.4 Overdraft Impossibility - Why You Can't Spend More Than You Have

#### The Question
"Can you spend more AGORA than you have in your account?"

#### The Answer: ABSOLUTELY IMPOSSIBLE âŒ

This is guaranteed by multiple layers of protection, making overdraft mathematically impossible, not just "very difficult."

#### Layer 1: Solana Token Program (Unbreakable)
```rust
// Solana's built-in token program checks BEFORE any transfer
token::transfer(ctx, amount)?;

// What happens internally:
pub fn transfer(from: Account, to: Account, amount: u64) {
    // AUTOMATIC CHECK by Solana (not our code)
    if from.balance < amount {
        return Err(TokenError::InsufficientFunds);  // STOPS HERE
    }
    
    // Only executes if sufficient balance
    from.balance -= amount;
    to.balance += amount;
}
```

#### Layer 2: Type System Protection
```rust
// All amounts are u64 (unsigned 64-bit integers)
pub fn transfer(amount: u64)  // CANNOT be negative

// Attempting negative amount:
transfer(-100)  // COMPILATION ERROR - won't even build
```

#### Layer 3: Atomic Transaction Model
```
Scenario: Alice has 100 AGORA, tries to send 150
Step 1: Check balance (100 < 150) âŒ
Step 2: ENTIRE TRANSACTION ROLLED BACK
Result: Alice keeps 100, Bob gets 0, nothing happens
```

#### Attack Scenarios - All Fail

##### Attack 1: Integer Overflow Attempt
```rust
// Attacker tries to overflow the balance
balance: 100 AGORA
send_amount: u64::MAX  // 18,446,744,073,709,551,615

// Result: InsufficientFunds error
// Solana checks ACTUAL balance, not mathematical tricks
```

##### Attack 2: Double Spend (Race Condition)
```rust
// User sends same 100 AGORA in two simultaneous transactions
Balance: 100 AGORA

Transaction 1: Send 100 to Bob    // Submitted at 12:00:00.000
Transaction 2: Send 100 to Carol  // Submitted at 12:00:00.000

// Solana processes sequentially (even if submitted simultaneously):
TX1: 100 - 100 = 0 âœ“  // Success, balance now 0
TX2: 0 - 100 = ERROR âŒ  // Fails, insufficient funds
```

##### Attack 3: Client Modification
```javascript
// Attacker modifies frontend JavaScript
original: await transfer(100);
modified: await transfer(1000000);

// Result: Transaction fails at blockchain
// Frontend is just UI - blockchain enforces rules
```

##### Attack 4: Smart Contract Bug Exploitation
```rust
// Even if our contract had a bug:
pub fn buggy_function() {
    // Accidentally tries to transfer without checking
    transfer_unchecked(user, 1_000_000);  // BUG!
}

// Still fails because:
// 1. Token Program checks balance
// 2. Account doesn't have 1,000,000
// Result: Transaction reverts
```

##### Attack 5: Negative Balance Trick
```rust
// Mathematical attempt to create negative
balance: 10 AGORA
subtract: 20 AGORA
result: -10  // IMPOSSIBLE with u64

// u64 cannot represent negative numbers
// 10 - 20 in u64 = Integer Underflow Error âŒ
```

##### Attack 6: Mint Authority Hijack
```rust
// Attacker tries to mint tokens to themselves
mint_to(attacker_wallet, 1_000_000);

// Fails because:
// - Only protocol_state PDA has mint authority
// - PDA requires specific seeds ["protocol"]
// - Cannot forge PDA signature
// Result: Unauthorized error âŒ
```

#### Fee Edge Cases

##### Scenario: Exact Balance with Fees
```rust
Balance: 100.00 AGORA
Send: 100.00 AGORA
Fee: 0.05 AGORA (0.05%)
Total needed: 100.05 AGORA

// Transaction FAILS - need 100.05 but have 100.00
// Maximum sendable: 99.95 AGORA
```

##### Solution in Our Contract
```rust
pub fn calculate_max_transfer(balance: u64, fee_rate: u64) -> u64 {
    // Returns maximum that can be sent INCLUDING fees
    balance * FEE_DIVISOR / (FEE_DIVISOR + fee_rate)
}

// Example: 100 AGORA with 0.05% fee
// Max transfer = 100 * 10000 / 10005 = 99.95 AGORA
```

#### Protection Statistics
| Protection Layer | Bypass Possibility | Security Level |
|-----------------|-------------------|----------------|
| Solana Token Program | 0% | Cryptographic |
| Type System (u64) | 0% | Compile-time |
| Atomic Transactions | 0% | Protocol level |
| Account Model | 0% | Blockchain core |
| Our Validations | 0% | Additional safety |

#### Mathematical Proof
```
Theorem: Balance can never go negative
Proof:
1. Let B = current balance (u64 â‰¥ 0)
2. Let T = transfer amount (u64 â‰¥ 0)
3. Transfer succeeds iff B â‰¥ T
4. New balance B' = B - T
5. Since B â‰¥ T and both are u64:
   B' â‰¥ 0 always
   
QED: Balance remains non-negative âˆŽ
```

#### Fun Facts

**Q: What if quantum computers break encryption?**
A: They'd break the signature, not the balance check. You still can't spend what doesn't exist.

**Q: What if someone controls 51% of validators?**
A: Solana uses Proof of Stake, not Proof of Work. Even with consensus control, you can't violate token program rules.

**Q: Has any Solana token ever had an overdraft bug?**
A: No. The token program has been battle-tested since 2020 with trillions in volume.

**Q: What about flash loans?**
A: Flash loans must be repaid in the same transaction. Balance checks still apply at each step.

#### Comparison to Traditional Banking
| System | Overdraft Possible | Protection |
|--------|-------------------|------------|
| Traditional Bank | Yes âœ“ | Policy (can be overridden) |
| Credit Card | Yes âœ“ | Credit limit (soft) |
| AGORA on Solana | No âŒ | Mathematics (unbreakable) |

#### Conclusion
**Spending more AGORA than you own is not just "very difficult" or "highly unlikely" - it is MATHEMATICALLY IMPOSSIBLE.**

This is the fundamental promise of blockchain: rules enforced by mathematics, not trust.

### 12.5 Upgrade Path

```rust
// All upgrades require DAO approval
pub fn upgrade_program(
    ctx: Context<UpgradeProgram>,
    new_program_id: Pubkey,
    migration_plan: MigrationPlan
) -> Result<()> {
    require!(
        ctx.accounts.proposal.status == ProposalStatus::Passed,
        ErrorCode::ProposalNotPassed
    );
    require!(
        ctx.accounts.proposal.yes_votes > ctx.accounts.proposal.no_votes * 3, 
        ErrorCode::InsufficientMajority  // 75% supermajority
    );
    // Execute upgrade...
}
```

---

## 13. Implementation Roadmap

### Phase 1: Foundation (Months 1-2) âœ…
- [x] Technical architecture
- [x] Economic modeling
- [x] Manifesto v1.1
- [x] Landing page
- [x] Treasury dashboard
- [ ] Smart contract development

### Phase 2: Development (Months 3-4)
- [ ] Complete Rust smart contracts
- [ ] Civic Pass integration
- [ ] Frontend application
- [ ] Comprehensive testing suite
- [ ] Devnet deployment

### Phase 3: Audit & Testing (Month 5)
- [ ] Internal code review
- [ ] Community bug bounty
- [ ] Professional audit (OtterSec/Neodyme)
- [ ] Stress testing on devnet
- [ ] Fix discovered issues

### Phase 4: Launch (Month 6)
- [ ] Mainnet deployment
- [ ] Initial contributor funding round
- [ ] User registration opens
- [ ] First retroactive claims
- [ ] Daily claiming begins

### Phase 5: Growth (Months 7-12)
- [ ] 10,000+ users onboarded
- [ ] First DAO proposals
- [ ] Merchant adoption program
- [ ] Mobile app development
- [ ] Cross-chain research

---

## 14. Technical Requirements

### 10.1 Infrastructure

**RPC Nodes:**
- Primary: GenesysGo or Helius
- Backup: Public Solana RPC
- Estimated cost: ~2 SOL/month

**Frontend Hosting:**
- GitHub Pages (free)
- Custom domain via Njalla
- CloudFlare CDN (optional)

**Data Storage:**
- On-chain: User accounts, governance
- Arweave: Historical data archives
- IPFS: Documentation and media

### 10.2 Development Stack

```json
{
  "blockchain": {
    "network": "Solana",
    "language": "Rust",
    "framework": "Anchor 0.29.0",
    "sdk": "@solana/web3.js"
  },
  "frontend": {
    "framework": "Vanilla JS",
    "wallet": "@solana/wallet-adapter",
    "identity": "@civic/solana-gateway",
    "ui": "Custom CSS"
  },
  "testing": {
    "unit": "cargo test",
    "integration": "anchor test",
    "frontend": "jest"
  },
  "monitoring": {
    "explorer": "Solscan API",
    "metrics": "Custom dashboard",
    "alerts": "Discord webhooks"
  }
}
```

### 10.3 Resource Estimation

**Transaction Costs:**
- User registration: ~0.01 SOL
- Daily claim: ~0.001 SOL
- Transfer: ~0.00025 SOL

**Storage Costs:**
- User account: ~0.00203 SOL (rent-exempt)
- Protocol state: ~0.01 SOL (rent-exempt)

**At 1M Users:**
- Total storage: ~2,030 SOL
- Daily transactions: ~1,000 SOL/year
- RPC costs: ~25 SOL/year

---

## Appendix A: Error Codes

```rust
#[error_code]
pub enum AgoraError {
    #[msg("User already registered")]
    AlreadyRegistered,
    
    #[msg("Invalid Civic Pass")]
    InvalidCivicPass,
    
    #[msg("Nothing to claim")]
    NothingToClaim,
    
    #[msg("Transfer locked (child account)")]
    TransferLocked,
    
    #[msg("Insufficient balance")]
    InsufficientBalance,
    
    #[msg("Invalid proposal")]
    InvalidProposal,
    
    #[msg("Voting period ended")]
    VotingEnded,
    
    #[msg("Already voted")]
    AlreadyVoted,
    
    #[msg("Unauthorized")]
    Unauthorized,
}
```

---

## Appendix B: Behavioral Scoring Algorithm

```rust
pub fn calculate_behavior_score(user: &UserState) -> BehaviorScore {
    let current_time = Clock::get()?.unix_timestamp;
    let days_since_registration = (current_time - user.registration_timestamp) / 86400;
    
    // Transaction frequency (transactions per day average)
    let tx_frequency = user.transaction_count as f64 / days_since_registration as f64;
    
    // Time since last transaction (staleness)
    let days_since_last_tx = (current_time - user.last_transaction_timestamp) / 86400;
    
    // Calculate scores (0-100 scale)
    let spending_score = min(100, (tx_frequency * 10.0) as u8);
    let hoarding_score = min(100, days_since_last_tx as u8);
    
    BehaviorScore {
        spending_frequency: spending_score,
        hoarding_tendency: hoarding_score,
        fee_multiplier: calculate_fee_multiplier(spending_score, hoarding_score),
    }
}

fn calculate_fee_multiplier(spending: u8, hoarding: u8) -> f64 {
    let base = 1.0;
    
    // Reward active spenders
    let spending_bonus = if spending > 80 { -0.2 } 
                        else if spending > 50 { -0.1 } 
                        else { 0.0 };
    
    // Penalize hoarders
    let hoarding_penalty = if hoarding > 80 { 0.5 }
                          else if hoarding > 50 { 0.25 }
                          else { 0.0 };
    
    (base + spending_bonus + hoarding_penalty).max(0.5).min(3.0)
}
```

---

## Appendix C: Retroactive Distribution Formula

```rust
pub fn calculate_initial_claim(age_in_days: u64) -> u64 {
    const DAILY_AMOUNT: u64 = 100_000_000_000; // 100 AGORA (9 decimals)
    const MAX_RETROACTIVE_DAYS: u64 = 365;
    
    if age_in_days >= MAX_RETROACTIVE_DAYS {
        // Adults: Full year retroactive
        MAX_RETROACTIVE_DAYS * DAILY_AMOUNT
    } else {
        // Children: Proportional to age
        age_in_days * DAILY_AMOUNT
    }
}
```

---

## Version History

- **v1.0.0** (Nov 20, 2025): Initial architecture with retroactive model
- **v1.1.0** (Nov 25, 2025): Added inflation model, behavioral scoring, updated governance  
- **v1.2.0** (Nov 26, 2025): Added Gas Pool Economics, sponsor tiers, attack mitigations
- **v1.3.0** (Nov 26, 2025): Added Merchant Economy, auto-detection, DAO incentives
- **v1.4.0** (Nov 28, 2025): **Gas Pool IMPLEMENTED** - Full implementation in lib.rs v3.4:
  - GasPoolState, SponsorRecord, TransactionTracker structs
  - 6 core functions (initialize, sponsor, claim, record, pause, resume)
  - 7 helper functions for tier calculation and anti-abuse
  - Anti-abuse system (ping-pong, circular, dust, rapid fire detection)
  - Emergency brake with auto-trigger
  - 6 new events, 6 new error codes
- **v1.5.0** (Nov 30, 2025): **DAO Governance IMPLEMENTED** - Full implementation in lib.rs v3.6:
  - Proposal system with 4 types (Standard, Treasury, Constitutional, Sanction)
  - Bond mechanism (20K-100K AGORA) preventing spam attacks
  - Automatic reputation system based on voting results
  - Bond multiplier formula: 1 + (abs(reputation) / 2)
  - VoteRecord PDA preventing double voting
  - Country sanctions system with reduced (never zero) UBI
  - Annual liveness verification with biometric proof of life
  - ProposalRegistry for global stats tracking

---

**For technical questions:** GitHub Issues  
**For general questions:** Discord  
**For security concerns:** security@agora-protocol.org (PGP available)

---

*"Code is law, but community is justice."*

**END OF DOCUMENT**
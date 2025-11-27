# AGORA Biometric Security System

## Overview

AGORA uses biometric deduplication to prevent one person from creating multiple accounts.

**Key principle:** Only INDIVIDUALS who commit fraud are punished. NO collective punishment of countries - this would be an attack vector.

---

## 1. Biometric Identity Verification

### EU eIDAS Requirements

EU Regulation 2019/1157 mandates that all EU identity cards contain:

| Biometric Data | Specification |
|----------------|---------------|
| **Fingerprints** | Both index fingers (left + right) |
| **Facial image** | Digital photograph |
| **Storage** | Secure chip on ID card |

Source: EU Regulation (EC) No 2252/2004, updated by Regulation (EU) 2019/1157

### How AGORA Uses This

```
REGISTRATION FLOW:

1. User scans both index fingers
2. eIDAS verifies: "This is [Name], identity confirmed" ✓
3. System creates: hash(left_index + right_index) → 0x7a3f...
4. Hash stored on Solana blockchain (as PDA seed)
5. Account created

DUPLICATE CHECK:

6. Another registration attempt with same fingers
7. hash(left_index + right_index) → 0x7a3f... (SAME!)
8. Blockchain: PDA already exists → Transaction FAILS
9. Duplicate REJECTED automatically by Solana
```

### Why This Prevents Fraud

| Attack | Why It Fails |
|--------|--------------|
| Use different fingers | eIDAS requires EXACTLY both index fingers - other fingers rejected |
| Register 10 times with 10 fingers | Impossible - must always use same 2 fingers |
| Use someone else's fingers | eIDAS verifies identity matches the fingers |
| Create ghost accounts | No fingers = no eIDAS verification = no account |
| Government issues fake IDs | Same fingerprints = same hash = blocked |

### Privacy Protection

```
What we store:     hash(fingerprints) → 0x7a3f...
What we DON'T:     actual fingerprint images

The hash is ONE-WAY:
- Cannot reverse hash to get fingerprints
- Can only check: "Does this new scan match existing hash?"
```

---

## 2. Why NO Country-Level Penalties

### The Problem with Collective Punishment

Country-level penalties create an **attack vector**:

```
ATTACK SCENARIO:

1. Attacker generates fake registrations from Country X
2. System detects "fraud from Country X"
3. System punishes ALL users from Country X
4. Legitimate users lose UBI through no fault of their own
5. Attacker has successfully sabotaged AGORA in that country

Cost to attacker: Low (just fake registrations)
Damage to AGORA: High (entire country affected)
```

### Our Solution: Individual Accountability Only

```
FRAUD DETECTION:

1. Duplicate biometric detected → Individual account banned
2. Fake identity discovered → Individual account banned
3. Pattern of fraud from same wallet → Individual account banned

NEVER:
- Penalize innocent users for others' fraud
- Reduce UBI based on country of origin
- Block registrations by country
```

### Why Individual Punishment Works

| Scenario | Response |
|----------|----------|
| Person tries duplicate account | That person banned, others unaffected |
| Organized fraud ring | Each fraudster banned individually |
| Government issues fake IDs | Fake IDs get same hash → still blocked |
| Million fake registrations | Each fails independently at PDA level |

---

## 3. Edge Cases

### Twins (Identical)

```
Problem:   Identical twins have same DNA but DIFFERENT fingerprints
Solution:  Fingerprint hashes will be different → Both can register ✓
```

### Lost Fingers

```
Problem:   User loses one or both index fingers after registration
Solution:
  1. Re-verification with remaining biometrics
  2. Government attestation of same identity
  3. Update hash with new biometric (e.g., middle fingers)
  4. Old hash linked to new hash on-chain
```

### Dead Person Fraud (Annual Liveness Check)

```
Problem:   Someone uses a dead person's eID card
Solution:
  1. Annual liveness verification required
  2. User must perform LIVE biometric scan
  3. Live scan must match eID biometrics
  4. Dead person cannot perform live scan → Account suspended
```

---

## 4. Technical Implementation

### Hash Generation

```rust
pub fn generate_biometric_hash(
    left_index: &BiometricTemplate,
    right_index: &BiometricTemplate,
) -> [u8; 32] {

    // Combine both fingerprints
    let combined = [left_index.as_bytes(), right_index.as_bytes()].concat();

    // Generate SHA-256 hash
    let hash = sha256(&combined);

    hash
}
```

### On-Chain Storage

```rust
// Biometric entry - PDA seeded by hash
// If hash exists, init fails = duplicate blocked
pub struct BiometricEntry {
    pub biometric_hash: [u8; 32],
    pub user_account: Pubkey,
    pub registered_at: i64,
    pub is_active: bool,
    pub bump: u8,
}

pub struct UserState {
    pub biometric_hash: [u8; 32],      // Cannot be reversed
    pub registration_timestamp: i64,
    pub last_claim_timestamp: i64,
    pub violations: u8,
    pub suspended_until: i64,
    pub permanently_blacklisted: bool,  // Individual fraud only
    // ... other fields
}
```

### Deduplication via PDA

```rust
#[derive(Accounts)]
#[instruction(biometric_hash: [u8; 32])]
pub struct RegisterUser<'info> {
    // ...

    // This is the magic - PDA seeded by biometric hash
    // If this hash already exists, init will FAIL
    // No additional code needed!
    #[account(
        init,
        payer = user,
        seeds = [b"biometric", biometric_hash.as_ref()],
        bump
    )]
    pub biometric_entry: Account<'info, BiometricEntry>,

    // ...
}
```

---

## 5. Summary

### What Makes This Secure

| Layer | Protection |
|-------|------------|
| **eIDAS** | Government verifies identity |
| **Biometric hash** | Prevents duplicate accounts |
| **PDA deduplication** | Blockchain enforces uniqueness |
| **Annual liveness** | Prevents dead person fraud |
| **Individual penalties** | Only fraudsters punished |

### Key Principles

1. **Individual accountability** - Only the fraudster is punished
2. **No collective punishment** - Countries are never penalized
3. **No attack vectors** - Cannot sabotage AGORA by framing a country
4. **Privacy preserved** - Only hashes stored, never biometrics
5. **Automatic enforcement** - Blockchain enforces rules, not committees

---

## 6. Document Info

**Version:** 2.0
**Last Updated:** November 2025
**Status:** Approved

**Changes from v1.0:**
- REMOVED: Country safeguard system (attack vector)
- ADDED: Explanation of why individual punishment only
- ADDED: Attack scenario for collective punishment

---

*"Same rules for everyone. Enforced by mathematics. Punish only the guilty."*

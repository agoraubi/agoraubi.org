# Security Policy

## Reporting a Vulnerability

Security is critical for AGORA Protocol. We take all security reports seriously.

### How to Report

**DO NOT** open a public GitHub issue for security vulnerabilities.

Instead, please email us at: **security@agoraubi.org**

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Any suggested fixes (optional)

### Response Timeline

| Action | Timeline |
|--------|----------|
| Initial response | Within 48 hours |
| Severity assessment | Within 7 days |
| Fix development | Depends on severity |
| Public disclosure | After fix is deployed |

### Severity Levels

| Level | Description | Example |
|-------|-------------|---------|
| **Critical** | Funds at risk | Token theft, infinite mint |
| **High** | Protocol integrity | Governance bypass |
| **Medium** | Limited impact | DoS vectors |
| **Low** | Minor issues | UI bugs, typos |

## Bug Bounty Program

Coming soon! We will reward responsible disclosure.

### In Scope

- Smart contracts (Rust/Anchor)
- Identity verification bypass
- Governance manipulation
- Economic exploits
- Token distribution bugs

### Out of Scope

- Social engineering
- Physical attacks
- Third-party services
- Already known issues

## Security Best Practices

### For Users

- Never share your private keys
- Verify transaction details before signing
- Use hardware wallets when possible
- Keep software updated

### For Developers

- All smart contracts undergo security audit
- Multi-sig required for critical operations
- Time-locks on governance changes
- Comprehensive test coverage

## Audit Status

| Component | Auditor | Status |
|-----------|---------|--------|
| Token Contract | TBD | Pending |
| Distribution | TBD | Pending |
| Governance | TBD | Pending |

---

**Thank you for helping keep AGORA secure!** 🔐

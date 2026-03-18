# ADR-007: Password reset via AWS SES

**Status**: done
**Date**: 2026-03-18

## Context

Users have no way to recover their account if they forget their password.

## Decision

Implement a password reset flow using time-limited tokens and AWS SES for email delivery.

## Implementation

### Phase 1: SES domain verification — partially automated

- [ ] Verify `tidyquote.app` domain in SES via AWS CLI (`aws ses verify-domain-identity`)
- [ ] CLI outputs DKIM tokens — add them as CNAME records in Cloudflare (manual)
- [ ] Add SPF TXT record for SES in Cloudflare (manual, if not already present)
- [ ] Set MAIL FROM domain via CLI (optional, improves deliverability)
- [ ] Wait for domain verification to complete

### Phase 2: Infrastructure — deployer role (aws-infrastructure repo) — automated

- [ ] Add SES permissions to deployer role: `ses:VerifyDomainIdentity`, `ses:GetIdentityVerificationAttributes` (for CI health checks)
- [ ] Add SES send permissions to Lambda execution role in `backend.yaml`: `ses:SendEmail`, `ses:SendRawEmail`
- [ ] Add `SES_SENDER` parameter to `backend.yaml` (default: `noreply@tidyquote.app`)
- [ ] Pass `SES_SENDER` and `SES_REGION` env vars to Lambda

### Phase 3: Backend (Rust) — automated

- [x] Add `verification_tokens` collection with `user_id`, `token_hash` (SHA-256), `expires_at` (1 hour), `used`, `purpose`
- [x] Create SES email client in infrastructure layer (implements `EmailSender` trait)
- [x] `POST /api/auth/forgot-password` — accepts email; generates random token; hashes and stores it; sends reset link via SES; always returns 200 (don't leak whether email exists)
- [x] `POST /api/auth/reset-password` — accepts token + new password; validates token (not expired, not used); updates password; marks token as used
- [x] Tests for token hashing

### Phase 4: Frontend (React) — automated

- [ ] "Forgot password?" link on login page
- [ ] `ForgotPasswordPage`: email input → calls `POST /api/auth/forgot-password` → shows "Check your email" message
- [ ] `ResetPasswordPage` at `/reset-password?token=...`: new password + confirm → calls `POST /api/auth/reset-password` → success → redirect to login
- [ ] Handle invalid/expired token with clear error message
- [ ] E2E test for forgot password flow (mock SES)

### Manual actions (you)

- [ ] Add DKIM CNAME records in Cloudflare (3 records, values provided by SES)
- [ ] Add SPF TXT record in Cloudflare if not present
- [ ] Request SES production access (AWS Support case) — sandbox only allows verified recipients

## Consequences

- SES sandbox limits sending to verified emails only — production access needed before real launch
- Reset tokens are single-use and expire after 1 hour
- Reuses SES infrastructure for email verification (ADR-008)
- `noreply@tidyquote.app` as sender — replies go nowhere (info@ is for general contact)

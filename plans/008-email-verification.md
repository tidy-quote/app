# ADR-008: Email verification on signup

**Status**: done
**Date**: 2026-03-18

## Context

Signups are not verified. Users can register with any email, including typos or fake addresses.

## Decision

Require email verification after signup, before the user can proceed to plan selection.

## Implementation

### Phase 1: Backend (Rust) — automated

- [x] Add `email_verified` field (default `false`) to user document
- [x] Add `verification_tokens` collection: `user_id`, `token_hash` (SHA-256), `expires_at` (24 hours), `used`, `purpose`
- [x] On signup: generate verification token; hash and store; send email via SES with link `{APP_BASE_URL}/verify?token=...`
- [x] `POST /api/auth/verify-email` — validates token; sets `email_verified: true`; marks token as used
- [x] `POST /api/auth/resend-verification` — authenticated; generates new token; sends new email
- [x] Gate access: unverified users get `403 { error: "email_not_verified" }` on pricing/quote endpoints
- [x] Tests for token hashing
- [ ] Rate limit resend to 1 per minute

### Phase 2: Frontend (React) — automated

- [x] After signup, redirect to `VerifyEmailPage`: "Check your email" message + resend button
- [x] `/verify?token=...` route: reads token from URL, calls verify endpoint, on success redirects to plan selection (ADR-003)
- [x] Show error if token is expired or invalid, with resend option
- [x] Update routing logic: signup → verify email → choose plan → app
- [ ] E2E test for verification flow (mock SES)

### Manual actions (you)

None — depends on SES being set up in ADR-007.

## Consequences

- Depends on SES domain verification (ADR-007 Phase 1)
- Adds a step between signup and payment — may increase drop-off, but prevents fake accounts
- Full signup flow: signup → verify email → choose plan (Stripe) → app

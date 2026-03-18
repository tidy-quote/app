# ADR-005: Usage tracking and quota enforcement

**Status**: accepted
**Date**: 2026-03-18

## Context

The pricing page advertises per-tier limits (Starter: 5/mo, Solo: 75/mo, Pro: unlimited) but nothing enforces them.

## Decision

Track quote generation count per user per billing period. Enforce limits before processing a lead.

## Implementation

### Phase 1: Backend (Rust) — all automated

- [ ] Define tier limits as constants: `STARTER_QUOTA = 5`, `SOLO_QUOTA = 75`, `PRO_QUOTA = None` (unlimited)
- [ ] Map Stripe price ID → tier → quota limit
- [ ] Add `usage` collection in MongoDB: `user_id`, `period_start`, `period_end`, `quote_count`
- [ ] On `checkout.session.completed` and `customer.subscription.updated` webhooks, store `current_period_start` and `current_period_end` from Stripe subscription
- [ ] Before processing a lead: check quote_count against tier limit; return `429 { error: "quota_exceeded", used: N, limit: N }` if exceeded
- [ ] After successful quote generation: increment `quote_count`
- [ ] `GET /api/usage` — return `{ used, limit, period_end }` for authenticated user
- [ ] Tests for quota enforcement, period reset, unlimited tier bypass

### Phase 2: Frontend (React) — all automated

- [ ] Call `GET /api/usage` on dashboard load
- [ ] Show usage bar on dashboard (e.g. "3 / 5 quotes this month")
- [ ] Show warning state when >= 80% used
- [ ] Show "Limit reached — upgrade your plan" CTA when exhausted, linking to plan selection
- [ ] Disable "New Quote" button when quota exceeded
- [ ] E2E test for usage display

### Manual actions (you)

None — fully automated.

## Consequences

- Quota enforcement depends on subscription status (ADR-003)
- "Unlimited" for Pro means no check, not a high number
- Billing period is aligned with Stripe subscription cycle
- Usage resets automatically when Stripe renews the subscription (webhook updates period)

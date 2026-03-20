# 012 — Setup-First Subscription Gating

**Status:** done
**Date:** 2026-03-20

## Context

Users couldn't save pricing templates because `authorize_user` enforced subscription checks on ALL protected endpoints. The intended model is "setup-first, pay to generate": users explore freely (pricing setup, dashboard, quote history) and only hit a paywall when generating a new quote.

## Decision

Split authorization into authentication (user exists, email verified, token valid) and subscription gating (active subscription required). Only the quote generation endpoint requires a subscription.

### Backend

- Renamed `authorize_user` → `authenticate_user` (checks: user exists, email verified, token not revoked)
- Added `require_subscription` — returns 403 `subscription_required` if not active
- Auth-only endpoints: save/get pricing, list/get quotes, get usage
- Auth + subscription: submit lead (quote generation) only
- Mirrored split in dev server (`authenticate_user_dev` + `require_subscription_dev`)

### Frontend

- Added `SubscriptionStatus` type and exposed it via `AuthContext`
- `SubscriptionRoute` component gates `/quote/new` — redirects to `/choose-plan` if inactive
- Dashboard, pricing setup, and quote history remain freely accessible
- Defense-in-depth: API client redirects to `/choose-plan` on 403 `subscription_required`
- `CheckoutSuccessPage` refreshes subscription context after successful payment
- Fixed PricingSetupPage error banner: errors now show red (`error-banner`) instead of green (`success-banner`)

## Implementation

- [x] Backend: split `authorize_user` into `authenticate_user` + `require_subscription`
- [x] Backend: mirror split in dev server
- [x] Backend: tests, clippy, fmt pass
- [x] Frontend: `SubscriptionStatus` type in domain
- [x] Frontend: subscription status in auth context via `useSyncExternalStore`
- [x] Frontend: `SubscriptionRoute` component
- [x] Frontend: wrap only `/quote/new` with `SubscriptionRoute`
- [x] Frontend: 403 fallback in API client
- [x] Frontend: fix PricingSetupPage error banner color
- [x] Frontend: refresh subscription after checkout
- [x] Frontend: lint and build pass

## Consequences

- Unsubscribed users can fully set up their pricing before paying
- Subscription check is only enforced where value is generated (quote creation)
- Adding new gated endpoints requires explicit `require_subscription` call — secure by default

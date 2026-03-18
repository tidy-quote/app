# ADR-003: Payments & subscriptions with Stripe Checkout

**Status**: done
**Date**: 2026-03-18

## Context

The app advertises three paid tiers (Starter $1.99/mo, Solo $8.99/mo, Pro $19.99/mo) but has no payment integration. Users must pay before accessing the app.

## Decision

Use Stripe Checkout (hosted) for payment. After signup + email verification, users are redirected to Stripe to pick a plan. Access is gated until an active subscription exists.

## Implementation

### Phase 1: Stripe setup (automated — `stripe` CLI)

- [x] Create 3 Products via `stripe products create` (Starter, Solo, Pro)
- [x] Create 3 recurring Prices via `stripe prices create` ($1.99, $8.99, $19.99 /month)
- [x] Create webhook endpoint via `stripe webhook_endpoints create` for `checkout.session.completed`, `customer.subscription.updated`, `customer.subscription.deleted`
- [x] Save price IDs and webhook secret as GitHub Actions secrets

### Phase 2: Infrastructure — deployer role (aws-infrastructure repo)

- [x] Add `sns:CreateTopic`, `sns:Subscribe`, `sns:Publish` to deployer role (needed for ADR-011 too)
- [x] No additional deployer permissions needed — Lambda role changes are already allowed

### Phase 3: Infrastructure — backend.yaml

- [x] Add parameters: `StripeSecretKey`, `StripeWebhookSecret`, `StripePriceStarter`, `StripePriceSolo`, `StripePricePro`
- [x] Pass new env vars to Lambda function
- [x] Add GitHub Actions secrets for all Stripe values

### Phase 4: Backend (Rust)

- [x] Add `stripe` crate dependency (or use raw HTTP — Stripe REST API is simple)
- [x] Add `stripe_customer_id`, `subscription_status`, `subscription_plan` fields to user document in MongoDB
- [x] Create Stripe infrastructure client (implements a `PaymentProvider` trait)
- [x] `POST /api/checkout` — authenticated; creates Stripe Checkout session with user's email + selected price ID; returns `{ url }` for redirect
- [x] `POST /api/webhook/stripe` — unauthenticated; verifies Stripe signature; handles:
  - `checkout.session.completed` → set subscription active, store customer ID + plan
  - `customer.subscription.updated` → update plan/status
  - `customer.subscription.deleted` → set subscription inactive
- [x] Add subscription guard middleware — all `/api/quote`, `/api/pricing` endpoints require `subscription_status == active`
- [x] Return `403` with `{ error: "subscription_required" }` when subscription is inactive
- [x] Tests for webhook signature verification, subscription guard, checkout session creation

### Phase 5: Frontend (React)

- [x] Create `ChoosePlanPage` — shown after email verification; 3 plan cards, each calls `POST /api/checkout` then redirects to Stripe URL
- [x] Create `CheckoutSuccessPage` — Stripe redirects here; polls for subscription status, then redirects to dashboard
- [x] Update routing: unverified → verify page; verified but no subscription → choose plan; active subscription → app
- [ ] Update `ProtectedRoute` to check subscription status from auth context
- [ ] Show subscription status on dashboard (plan name, renewal date)
- [x] Handle cancelled/expired: block app access with "Resubscribe" CTA that calls `POST /api/checkout`
- [ ] E2E test for the choose-plan → checkout-success flow (mock Stripe redirect)

### Manual actions (you)

- [x] Install Stripe CLI if not already (`brew install stripe/stripe-cli/stripe` or equivalent)
- [x] Run `stripe login` to authenticate
- [x] Add Stripe secret key, webhook secret, and price IDs to GitHub Actions secrets

## Consequences

- Users cannot use the app without paying
- Stripe manages billing, invoices, card updates, and failed payments
- Webhook is the source of truth for subscription status
- Need to handle race condition: user completes Checkout but webhook hasn't arrived yet (CheckoutSuccessPage polls)

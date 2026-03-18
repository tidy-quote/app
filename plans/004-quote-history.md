# ADR-004: Persist and display quote history

**Status**: done
**Date**: 2026-03-18

## Context

Quotes are generated but never saved. Users lose all generated quotes when they leave the page.

## Decision

Persist quotes in MongoDB and add a history view.

## Implementation

### Phase 1: Backend (Rust) — all automated

- [x] Define `Quote` domain entity: id, user_id, lead_text, job_summary, price_breakdown, follow_up_message, tone, created_at
- [x] Add `quotes` collection to `MongoStore` (implements a `QuoteStore` trait)
- [x] Create MongoDB index on `(user_id, created_at)` for efficient listing
- [x] Save quote in `process_lead` use case after successful generation
- [x] `GET /api/quotes` — list quotes for authenticated user (newest first, paginated with `?page=1&limit=20`)
- [x] `GET /api/quotes/:id` — fetch single quote for authenticated user
- [x] Tests for quote persistence, listing, pagination, ownership check

### Phase 2: Frontend (React) — all automated

- [ ] Add API client methods: `getQuotes(page)`, `getQuote(id)`
- [ ] Replace dashboard quick actions with recent quotes list
- [ ] Quote list item: lead preview (first ~60 chars), total price, date
- [ ] Quote detail view: full job summary, price breakdown, follow-up message with copy button
- [ ] Loading skeleton, empty state ("No quotes yet — create your first one")
- [ ] Update bottom nav: replace "Dashboard" with "Quotes" (or add a 4th tab)
- [ ] E2E test for quote generation → appears in history → detail view

### Manual actions (you)

None — fully automated.

## Consequences

- Quotes are persisted and retrievable
- MongoDB storage grows over time — may need TTL or archival later
- Dashboard becomes more useful with recent activity

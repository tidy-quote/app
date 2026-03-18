# ADR-009: Request body validation on backend

**Status**: done
**Date**: 2026-03-18

## Context

The backend only does JSON deserialization — malformed or oversized payloads are not rejected early.

## Decision

Add schema validation at the presentation layer for all API endpoints using serde and custom validation logic.

## Implementation

### Phase 1: Backend (Rust) — all automated

- [x] Define validation constraints as constants in a `validation` module:
  - Email: max 254 chars, valid format
  - Password: 8–72 chars
  - Lead text: 1–10,000 chars
  - Images: max 5 per request, max 5MB base64 each
  - Pricing template: max 50 categories, max 50 add-ons
  - Category/add-on name: max 100 chars
  - Price values: 0–99,999
  - Tone: must be one of `friendly`, `direct`, `premium`
- [x] Add `validate()` method to each request DTO (returns `Result<(), ValidationError>`)
- [x] Call `validate()` in handlers before passing to use cases
- [x] Return `400 { error: "..." }` with a descriptive message for each validation failure
- [x] Add API Gateway payload size limit to `backend.yaml` (10MB max, matching Lambda limit)
- [x] Tests for every validation rule (valid + invalid cases)

### Manual actions (you)

None — fully automated.

## Consequences

- Rejects bad input before it reaches the AI client or database
- Prevents oversized payloads from consuming Lambda memory/time
- No new dependencies — serde + custom validators

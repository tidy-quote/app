# ADR-006: API rate limiting via API Gateway

**Status**: done
**Date**: 2026-03-18

## Context

The API has no protection against abuse or brute-force attacks.

## Decision

Use API Gateway throttling to apply global rate limits.

## Implementation

### Phase 1: backend.yaml — automated

- [ ] Add `ThrottlingBurstLimit` and `ThrottlingRateLimit` to `HttpApiStage` default route settings (e.g. 100 burst, 50 rate)
- [ ] Add a dedicated route for `/api/auth/{proxy+}` with stricter throttling (e.g. 10 burst, 5 rate)
- [ ] API Gateway returns `429 Too Many Requests` automatically when throttled

### Phase 2: deployer role (aws-infrastructure repo) — automated

- [ ] No changes needed — deployer already has full `apigateway:*` on the API

### Manual actions (you)

None — fully automated.

## Consequences

- Coarse-grained (per-IP, not per-user) but sufficient as a safety net
- Per-user quota enforcement is handled separately in ADR-005
- No additional infrastructure or code changes needed
- HTTP API v2 supports route-level throttling via stage default and route overrides

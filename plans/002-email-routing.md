# ADR-002: Email routing for info@tidyquote.app

**Status**: done
**Date**: 2026-03-18

## Context

The app needs a professional contact address (`info@tidyquote.app`) for customer communication. A full mailbox is overkill at this stage — forwarding to a personal inbox is sufficient.

## Decision

Use Cloudflare Email Routing (free) to forward `info@tidyquote.app` to a personal email address.

## Implementation

- [x] Enable Email Routing on Cloudflare (`tidyquote.app` zone)
- [x] Add personal email as verified destination
- [x] Create route: `info@tidyquote.app` → personal email
- [x] Verify MX and SPF DNS records were auto-added
- [x] Send test email to `info@tidyquote.app` and confirm delivery

## Consequences

- Receive-only: replies come from personal address unless a "Send As" alias is configured later
- Can upgrade to a full mailbox (Google Workspace, Zoho) if needed without changing the public address

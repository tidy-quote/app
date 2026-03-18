# ADR-001: Connect custom domains on Cloudflare

**Status**: done
**Date**: 2026-03-18

## Context

The app was deployed but all services used auto-generated URLs.
The domain `tidyquote.app` was already on Cloudflare (active zone).

## Decision

Map custom subdomains via Cloudflare DNS:

| Subdomain | Target | Purpose |
|---|---|---|
| `tidyquote.app` | GitHub Pages | Landing page |
| `app.tidyquote.app` | AWS Amplify (CloudFront) | React SPA |
| `api.tidyquote.app` | API Gateway (custom domain) | Backend API |

## Implementation

- [x] Add `docs/CNAME` file with `tidyquote.app`
- [x] Add ACM cert + API Gateway custom domain to `backend.yaml`
- [x] Tighten CORS from `*` to `https://app.tidyquote.app`
- [x] Add ACM + domain name permissions to deployer role
- [x] Cloudflare DNS: root + www → GitHub Pages
- [x] Cloudflare DNS: app → Amplify CloudFront (`d2hug279zk1aot.cloudfront.net`)
- [x] Cloudflare DNS: api → API Gateway (`d-qs6rwwg9a4.execute-api.eu-west-1.amazonaws.com`)
- [x] Cloudflare DNS: ACM validation records for both certs
- [x] GitHub Pages custom domain + HTTPS enforced
- [x] Amplify custom domain association for `app.tidyquote.app`
- [x] `API_BASE_URL` secret updated to `https://api.tidyquote.app`

## Consequences

- All Cloudflare DNS records use proxy OFF (DNS only)
- CORS restricted to `https://app.tidyquote.app`
- Smoke tests will need a re-run (failed during deploy due to cert propagation timing)

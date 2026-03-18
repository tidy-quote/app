# ADR-001: Connect custom domains on Cloudflare

**Status**: in-progress
**Date**: 2026-03-18

## Context

The app is deployed but all services use auto-generated URLs:
- Landing page: `tidy-quote.github.io/app/` (GitHub Pages)
- App (SPA): `main.djvwq9ny14lwu.amplifyapp.com` (AWS Amplify)
- API: `<id>.execute-api.eu-west-1.amazonaws.com` (API Gateway)

The domain `tidyquote.app` is already on Cloudflare (active zone).

## Decision

Map custom subdomains via Cloudflare DNS:

| Subdomain | Target | Purpose |
|---|---|---|
| `tidyquote.app` | GitHub Pages | Landing page |
| `app.tidyquote.app` | AWS Amplify | React SPA |
| `api.tidyquote.app` | API Gateway (custom domain) | Backend API |

## Implementation

### Code changes

- [x] Add `docs/CNAME` file with `tidyquote.app`
- [x] Add ACM cert + API Gateway custom domain to `backend.yaml`
- [x] Tighten CORS from `*` to `https://app.tidyquote.app`
- [x] Update smoke test URL to `https://app.tidyquote.app`
- [x] Add ACM + domain name permissions to deployer role (aws-infrastructure)

### AWS

- [x] Deploy updated deployer role

### Cloudflare DNS

- [x] CNAME `tidyquote.app` → `tidy-quote.github.io` (proxy OFF)
- [x] CNAME `www` → `tidy-quote.github.io` (proxy OFF)
- [x] CNAME `app` → `main.djvwq9ny14lwu.amplifyapp.com` (proxy OFF)
- [ ] CNAME for ACM DNS validation (after first deploy)
- [ ] CNAME `api` → API Gateway regional domain (after first deploy)

### GitHub

- [x] Pages custom domain set to `tidyquote.app`
- [x] `API_BASE_URL` secret updated to `https://api.tidyquote.app`
- [ ] Enforce HTTPS (waiting for GitHub to issue cert, ~minutes)

### Post-deploy

- [ ] Push code, trigger deploy
- [ ] Add ACM validation CNAME in Cloudflare (from ACM console)
- [ ] Add `api` CNAME from `ApiCustomDomainTarget` stack output

## Consequences

- GitHub Pages requires Cloudflare proxy OFF (DNS only)
- First deploy will wait on ACM cert validation — add the DNS record promptly
- CORS is now restricted to `https://app.tidyquote.app`

# ADR-010: React error boundaries

**Status**: done
**Date**: 2026-03-18

## Context

The React app has no error boundaries. An unhandled error in any component crashes the entire app with a white screen.

## Decision

Add error boundaries at key layout points to catch rendering errors and show a fallback UI.

## Implementation

### Phase 1: Frontend (React) — all automated

- [x] Create `ErrorBoundary` class component with `componentDidCatch` logging
- [x] Create `ErrorFallback` functional component: "Something went wrong" message + "Try again" button (resets error state)
- [x] Wrap `<Outlet />` inside `AppLayout` with `ErrorBoundary` — catches page-level errors without losing nav
- [x] Wrap root `<App />` in `main.tsx` with a top-level `ErrorBoundary` — last-resort fallback: "App error" + "Reload" button
- [x] Style fallback UI consistent with app design (uses existing CSS variables)
- [x] Tests: render a component that throws, verify fallback appears

### Manual actions (you)

None — fully automated.

## Consequences

- Errors are contained — a broken page doesn't take down the whole app
- Users see a recovery action instead of a blank screen
- Class component required by React for error boundaries (no hook equivalent)

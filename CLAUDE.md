# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Philosophy

Code like Kent Beck. Prioritize simplicity, readability, and testability. Make the code clear enough that it doesn't need comments to explain what it does.

### Working Style

- **Proceed step by step.** Keep code changes small and incremental so each implementation detail can be reviewed before moving on.
- **Understand before changing.** Read existing code before modifying it. Understand the context, patterns, and constraints already in place.
- **Minimal changes.** Make the smallest change that solves the problem. Avoid refactoring unrelated code, adding unnecessary abstractions, or "improving" things that weren't asked for.

## Project Overview

**Tidy-Quote** is an AI SaaS that helps solo residential cleaners turn messy inbound requests into clear quote drafts and follow-up messages.

Core workflow: cleaner pastes a customer message or uploads a photo → AI extracts job details → generates a draft quote using the cleaner's pricing rules → produces a professional follow-up message.

### Tech Stack

- **Frontend**: React + TypeScript (single page application, mobile-first PWA)
- **Backend**: Rust (deployed as AWS Lambda functions)
- **Database**: MongoDB
- **AI Provider**: OpenAI-compatible API (OpenRouter or Groq), provider-agnostic by design
- **E2E Tests**: Playwright

### Default AI Configuration

- Provider: OpenRouter
- Model: `google/gemini-3-flash-preview`
- The AI client is provider-agnostic: swap base URL + API key to switch between OpenRouter, Groq, or any OpenAI-compatible provider.

## Architecture

### Frontend (React + TypeScript)

```
frontend/
├── src/
│   ├── domain/           # Core types, interfaces, business logic
│   ├── application/      # Use cases, state management, API client
│   ├── infrastructure/   # HTTP clients, storage adapters
│   └── presentation/     # React components, pages, hooks
├── e2e/                  # Playwright tests
└── public/
```

### Backend (Rust Lambdas)

```
backend/
├── src/
│   ├── domain/           # Entities, value objects, business rules
│   ├── application/      # Use cases, ports (traits)
│   ├── infrastructure/   # MongoDB adapter, AI client, external services
│   └── presentation/     # Lambda handlers, request/response types
└── tests/
```

### Layer Rules

1. **Domain layer** (innermost)
   - Business entities: `PricingTemplate`, `ServiceCategory`, `Quote`, `Lead`, `JobSummary`
   - Value objects: `UserId`, `TemplateId`, `QuoteId`
   - Pure business logic: pricing calculations, quote generation from rules
   - NO dependencies on other layers or external crates (except std + serde)

2. **Application layer**
   - Defines ports (traits): `trait LeadProcessor`, `trait PricingStore`, `trait AiClient`
   - Contains use cases that orchestrate the workflow
   - Depends only on domain layer

3. **Infrastructure layer**
   - Implements application traits: MongoDB store, OpenAI-compatible AI client
   - Depends on application layer (for trait definitions)

4. **Presentation layer**
   - Lambda handlers / React components
   - Wires infrastructure implementations to application use cases
   - Error reporting to user

### Dependency Direction

```
presentation  → application → domain
infrastructure → application
```

## SOLID Principles

**S - Single Responsibility**: Each module/struct has one reason to change.

**O - Open/Closed**: Use traits (Rust) and interfaces (TypeScript) to allow new implementations without changing existing code. The AI provider is a prime example — swap implementations without touching use cases.

**L - Liskov Substitution**: Implementations must be substitutable for their trait/interface.

**I - Interface Segregation**: Prefer small, focused traits/interfaces.

**D - Dependency Inversion**: High-level modules depend on abstractions, not concrete implementations.

## Code Style

### Parse, Don't Validate

Encode invariants into the type system so invalid states are unrepresentable.

- Parse at the boundary, then trust the types
- Newtypes with fallible constructors in Rust
- Discriminated unions for state management in TypeScript
- Prefer semantic types over primitives

### TypeScript Conventions

- Strict mode always enabled
- Explicit return types on exported functions
- `interface` for object shapes, `type` for unions/intersections
- Functional components only, no class components
- Props interface named `{ComponentName}Props`

### Rust Conventions

- Use `Result<T, E>` for fallible operations
- `thiserror` for library errors, `anyhow` for Lambda handlers
- Newtype pattern for domain types
- `serde` for serialization with explicit derives
- `cargo fmt` and `cargo clippy` before committing

## Test-First Development

- **Domain logic**: Write the test before the implementation. Confirm it fails, then write the minimal code to make it pass.
- **Never add a function without a corresponding test.**
- **E2E tests**: Playwright for user-facing flows.
- **Unit tests**: Colocated with source code.

### Test Naming

Use direct, descriptive names without "should":
- `returns_null_when_user_not_found`
- `throws_error_for_invalid_input`

### Testing Commands

```bash
# Frontend
cd frontend && npm test              # Unit tests
cd frontend && npx playwright test   # E2E tests

# Backend
cd backend && cargo test             # All tests
cd backend && cargo clippy           # Linter
cd backend && cargo fmt --check      # Format check
```

## Error Handling

### Rust
- Define specific error enums per layer
- Use `thiserror` for derive macros
- Propagate errors with `?` operator
- Convert between error types at layer boundaries

### TypeScript
- Use discriminated unions for Result types
- Include context in error messages
- Handle errors at boundaries (API calls, user input)

## Build & Run Commands

```bash
# Frontend
cd frontend && npm install           # Install dependencies
cd frontend && npm run dev           # Development server
cd frontend && npm run build         # Production build
cd frontend && npm run lint          # Lint

# Backend
cd backend && cargo build            # Development build
cd backend && cargo build --release  # Release build
cd backend && cargo test             # Run tests
cd backend && cargo clippy           # Lint
cd backend && cargo fmt              # Format
```

## Post-Implementation Refactoring

After completing each implementation, always refactor the code:
- Remove dead code
- Simplify what can be simplified (redundant branches, nested conditions, duplicate logic)

## ADR Maintenance

After completing any task that relates to an ADR in `plans/`:
- Update the relevant ADR checklist — check off completed items, leave genuinely unfinished items unchecked
- If all items are done, set the status to `done`
- If work is partially complete, set the status to `in-progress`
- Never leave an ADR marked `done` with unchecked items, or marked `accepted` when work has started

## What NOT to Do

- Don't add features beyond what was asked
- Don't refactor unrelated code
- Don't add unnecessary comments, docstrings, or type annotations to unchanged code
- Don't add error handling for scenarios that can't happen
- Don't create abstractions for one-time operations
- Don't design for hypothetical future requirements

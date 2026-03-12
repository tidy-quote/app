# Specification: QuoteSnap

## 1. Product Summary

Build a lightweight AI SaaS that helps solo residential cleaners turn messy inbound requests into clear quote drafts and follow-up messages.

Product name: **QuoteSnap**

Core promise: **reply faster, quote more consistently, and win more cleaning jobs with less admin work.**

This product fits the original goal because it:
- uses AI in a practical way
- can be sold as a subscription starting at **$1.99/month**
- avoids heavy founder operations such as logistics, marketplaces, human experts, or regulated advice

## 2. Target Customer

Primary user:
- solo residential cleaners
- very small cleaning businesses with 1-3 people
- operators who acquire leads through WhatsApp, SMS, Instagram, Facebook, or contact forms

Initial geography:
- English-speaking markets first
- especially US, UK, Canada, Australia

Launch language scope:
- English only
- no translation features in MVP

Ideal customer characteristics:
- gets several new quote requests each week
- does pricing manually
- often replies late or inconsistently
- does not use a full CRM

## 3. Problem

Residential cleaners lose time and revenue because incoming leads are unstructured.

Typical lead messages look like:
- "Hi, how much for a deep clean tomorrow?"
- "Need move-out cleaning for a 2 bed apartment"
- photos with almost no context

Current pain points:
- the cleaner has to extract missing details manually
- quotes are inconsistent from one lead to another
- response time is slow when the cleaner is busy onsite
- some leads are lost because follow-up never happens

## 4. Proposed Solution

The app lets a cleaner paste a customer message or upload a photo or chat screenshot into a simple workflow. The system converts that input into:
- a structured job summary
- a list of missing details to ask for
- a draft quote based on the cleaner's pricing template
- a polished follow-up message ready to send manually

The AI is an assistant, not an autopilot.

Rules:
- it does not auto-send messages
- it does not finalize pricing without user review
- it does not guarantee job scope accuracy
- it uses the cleaner's own price rules as the source of truth

## 5. Jobs To Be Done

Main job:
- "When a lead arrives, help me turn it into a professional quote in under 2 minutes."

Supporting jobs:
- "Help me ask the right follow-up questions."
- "Keep my pricing consistent."
- "Make me sound professional even when I answer quickly."
- "Save me from rewriting the same quote messages every day."

## 6. MVP Scope

The MVP does only four things:
- lead intake
- structured job summary
- pricing template setup
- draft quote and follow-up generation

### 6.1 Must-Have Features

1. Lead intake
- paste plain text
- upload photos or chat screenshots

2. Structured job summary
- extract service type
- extract property size details when present
- extract requested date/time
- identify missing information
- produce a structured summary

3. Pricing template setup
- cleaner configures base prices
- cleaner configures common add-ons
- cleaner configures service categories such as standard clean, deep clean, move-out clean
- cleaner configures reusable pricing rules

4. Draft quote and follow-up generation
- creates a quote draft from extracted info plus stored pricing rules
- clearly shows assumptions
- flags missing inputs before final use
- creates a short clarification message if important details are missing
- creates a quote message when enough info is available
- supports tone options such as friendly, direct, premium

### 6.2 Explicit Non-Goals for MVP

- no direct WhatsApp integration
- no CRM pipeline
- no auto-sending
- no payment collection
- no scheduling or dispatch
- no route planning
- no invoicing
- no team collaboration beyond a single account
- no marketplace features
- no voice note transcription
- no multi-language output
- no saved FAQ library in MVP
- no quote history beyond what is needed for the current session

### 6.3 Later Expansion Options

- voice note transcription
- translation support
- direct channel integrations
- lightweight CRM pipeline views

## 7. User Flow

### 7.1 First-Time Setup

1. User signs up.
2. User chooses country and currency.
3. User fills a simple pricing wizard:
- service types
- base prices
- add-ons
- minimum callout price
- custom notes
4. User tests the system with an example lead.

### 7.2 Daily Usage

1. A new lead arrives on the cleaner's normal channel.
2. The cleaner opens QuoteSnap and pastes or uploads the lead.
3. The system returns:
- job summary
- missing questions
- draft quote
- suggested message
4. The cleaner edits if needed.
5. The cleaner manually copies the response back to WhatsApp or another channel.

## 8. Pricing Model

### Starter
- **$1.99/month**
- up to 5 AI quote generations per month
- one pricing template

### Solo
- **$8.99/month**
- up to 75 AI quote generations per month
- multiple service templates

### Pro
- **$19.99/month**
- unlimited reasonable use
- priority generations
- multi-location pricing templates

The low entry tier is mainly a friction remover. Real revenue should come from Solo and Pro.

## 9. Competitive Angle

This is not another generic AI writing tool.

The differentiation is:
- built for one specific service business
- tied to a revenue event: replying to leads
- uses business-specific pricing rules
- simple enough for non-technical operators

## 10. Why This Can Work

Reasons this concept is attractive:
- quote requests are frequent
- ROI is clear
- retention can be strong because leads keep arriving
- setup is simple
- the product can be self-serve
- the founder does not need to manage supply, logistics, or human experts

## 11. Business Model and Distribution

Primary acquisition channels:
- Facebook groups for cleaners
- TikTok and Instagram content showing before/after quote workflows
- direct outreach to solo cleaners
- partnerships with creators who teach cleaning businesses how to get clients

Activation hook:
- show "paste a lead, get a quote draft in seconds" immediately after signup

Retention hook:
- saved pricing rules make the app more valuable over time

## 12. Technical Approach

Recommended product shape:
- mobile-first web app
- PWA, no app store requirement
- simple auth
- lightweight dashboard

Core components:
- multimodal LLM for extraction and message drafting
- rules layer using user-defined pricing templates
- storage for customers and pricing templates

Optional later components:
- speech-to-text for voice notes
- quote history and exports

Important design constraint:
- pricing output should be generated from deterministic pricing rules where possible
- AI should help with extraction and wording, not be the only pricing engine

## 13. Founder Operations

This should stay easy to manage if built with these constraints:
- self-serve onboarding only
- no done-for-you quoting service
- no human review layer
- support limited to product questions
- no promises of quote accuracy beyond user-controlled templates
- no channel integrations in v1

## 14. Risks and Mitigations

Risk: bad quotes due to missing information
- Mitigation: show assumptions clearly and prompt for missing details before generating final draft

Risk: pricing varies a lot by region
- Mitigation: require user-owned pricing templates instead of global default pricing

Risk: users expect full automation
- Mitigation: position product as draft assistant, not autopilot

Risk: low willingness to pay from very small operators
- Mitigation: keep entry tier at $1.99 and prove value with time saved and faster replies

Risk: feature creep into full CRM software
- Mitigation: maintain strict non-goals for v1

## 15. Success Metrics

Primary metrics:
- time from signup to first generated quote
- weekly active quoting users
- number of quote drafts generated per active account
- trial-to-paid conversion
- paid retention after 60 and 90 days

Outcome metrics:
- reported time saved
- reply speed improvement
- user-reported booked jobs influenced by the tool

## 16. Validation Plan

Before building too much:
1. Build a landing page with a clear promise.
2. Show a fake or semi-manual demo to real cleaners.
3. Interview at least 15 solo cleaners.
4. Confirm that cleaners already receive enough quote requests to justify a subscription.
5. Validate whether copy-paste workflow is acceptable before attempting integrations.

## 17. Final Product Decision

Chosen concept:
- **AI quote assistant for solo residential cleaners**

Why this idea was selected:
- stronger ROI than consumer convenience apps
- lower liability than health-adjacent products
- lower founder overhead than marketplaces or service businesses
- easy to launch with a narrow MVP and low starting price

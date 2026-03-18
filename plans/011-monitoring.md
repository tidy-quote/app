# ADR-011: Monitoring and observability via CloudWatch

**Status**: done
**Date**: 2026-03-18

## Context

The backend logs to stderr but has no structured logging, no alarms, and no dashboards.

## Decision

Use CloudWatch for structured logging and alarms. No additional services.

## Implementation

### Phase 1: Backend (Rust) — automated

- [x] Add `tracing` and `tracing-subscriber` crates with JSON formatter
- [x] Initialize tracing subscriber in Lambda `main()` with JSON output
- [x] Add structured log fields to key events:
  - Auth: `event=signup|login|logout`, `user_id`, `success`
  - Quote: `event=quote_generated`, `user_id`, `tone`, `duration_ms`
  - Webhook: `event=stripe_webhook`, `type`, `customer_id`
  - Error: `event=error`, `endpoint`, `error_type`, `message`
- [ ] Include `request_id` from Lambda context in all log entries

### Phase 2: Infrastructure — deployer role (aws-infrastructure repo) — automated

- [x] Add CloudWatch alarm permissions to deployer role: `cloudwatch:PutMetricAlarm`, `cloudwatch:DeleteAlarms`, `cloudwatch:DescribeAlarms`
- [x] Add SNS permissions to deployer role: `sns:CreateTopic`, `sns:DeleteTopic`, `sns:Subscribe`, `sns:GetTopicAttributes`, `sns:SetTopicAttributes`
- [x] Add CloudWatch dashboard permissions: `cloudwatch:PutDashboard`, `cloudwatch:DeleteDashboards`, `cloudwatch:GetDashboard`

### Phase 3: Infrastructure — backend.yaml — automated

- [x] Add SNS topic `tidy-quote-alarms` with email subscription to `info@tidyquote.app`
- [x] Add CloudWatch alarms:
  - Lambda errors > 5 in 5 minutes
  - Lambda p99 duration > 20s
  - 5xx responses > 10 in 5 minutes
- [ ] Add CloudWatch dashboard with key metrics: invocations, errors, duration, 4xx/5xx rates

### Manual actions (you)

- [x] Confirm SNS email subscription (AWS sends a confirmation email to `info@tidyquote.app` — click the link)

## Consequences

- No extra cost beyond standard CloudWatch (included with Lambda)
- Alarms notify via email — no PagerDuty/Slack integration needed initially
- Structured JSON logs enable CloudWatch Insights queries for debugging
- SNS subscription requires one-time email confirmation

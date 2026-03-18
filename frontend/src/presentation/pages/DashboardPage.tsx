import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import { getPricingTemplate, getQuotes, getUsage, type UsageInfo } from "../../application/api";
import type { QuoteDraft } from "../../domain/types";
import "./DashboardPage.css";

function formatDate(iso: string): string {
  return new Date(iso).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
  });
}

function truncate(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text;
  return text.slice(0, maxLen).trimEnd() + "...";
}

export function DashboardPage(): React.JSX.Element {
  const [hasPricing, setHasPricing] = useState<boolean | null>(null);
  const [quotes, setQuotes] = useState<QuoteDraft[]>([]);
  const [loadingQuotes, setLoadingQuotes] = useState(true);
  const [usage, setUsage] = useState<UsageInfo | null>(null);

  useEffect(() => {
    getPricingTemplate()
      .then((template) => setHasPricing(template !== null))
      .catch(() => setHasPricing(false));

    getQuotes(1, 5)
      .then(setQuotes)
      .catch(() => setQuotes([]))
      .finally(() => setLoadingQuotes(false));

    getUsage()
      .then(setUsage)
      .catch(() => {});
  }, []);

  const quotaExhausted = usage !== null && usage.limit !== null && usage.used >= usage.limit;
  const quotaWarning = usage !== null && usage.limit !== null && usage.used >= usage.limit * 0.8 && !quotaExhausted;

  return (
    <div className="dashboard">
      <h2 className="dashboard-title">Welcome to Tidy-Quote</h2>
      <p className="dashboard-subtitle">
        Generate professional quotes from customer messages in seconds.
      </p>

      {hasPricing === false && (
        <div className="setup-cta">
          <p className="setup-cta__text">
            Set up your pricing template first so quotes can be generated with
            your rates.
          </p>
          <Link to="/pricing" className="setup-cta__link">
            Set Up Pricing
          </Link>
        </div>
      )}

      {usage && (
        <div className={`usage-bar${quotaWarning ? " usage-bar--warning" : ""}${quotaExhausted ? " usage-bar--exhausted" : ""}`}>
          <div className="usage-bar__label">
            <span>
              {usage.used} / {usage.limit ?? "Unlimited"} quotes this month
            </span>
          </div>
          {usage.limit !== null && (
            <div
              className="usage-bar__track"
              role="progressbar"
              aria-valuenow={usage.used}
              aria-valuemin={0}
              aria-valuemax={usage.limit}
              aria-label={`${usage.used} of ${usage.limit} quotes used this month`}
            >
              <div
                className="usage-bar__fill"
                style={{ width: `${Math.min(100, (usage.used / usage.limit) * 100)}%` }}
              />
            </div>
          )}
          {quotaExhausted && (
            <p className="usage-bar__cta">
              Limit reached.{" "}
              <Link to="/choose-plan" className="auth-link">
                Upgrade your plan
              </Link>
            </p>
          )}
        </div>
      )}

      <div className="quick-actions">
        {quotaExhausted ? (
          <div className="action-card action-card--primary action-card--disabled">
            <span className="action-card__title">New Quote</span>
            <span className="action-card__desc">
              Monthly quota reached — upgrade to generate more quotes
            </span>
          </div>
        ) : (
          <Link to="/quote/new" className="action-card action-card--primary">
            <span className="action-card__title">New Quote</span>
            <span className="action-card__desc">
              Paste a lead message or upload photos to generate a quote
            </span>
          </Link>
        )}

        <Link to="/pricing" className="action-card">
          <span className="action-card__title">Pricing Setup</span>
          <span className="action-card__desc">
            {hasPricing
              ? "View or update your service categories, add-ons, and rates"
              : "Configure your service categories, add-ons, and rates"}
          </span>
        </Link>
      </div>

      <section className="recent-quotes">
        <h3 className="recent-quotes__title">Recent Quotes</h3>

        {loadingQuotes && (
          <div className="recent-quotes__loading" role="status">Loading...</div>
        )}

        {!loadingQuotes && quotes.length === 0 && (
          <p className="recent-quotes__empty">
            No quotes yet — create your first one.
          </p>
        )}

        {!loadingQuotes && quotes.length > 0 && (
          <ul className="quote-list">
            {quotes.map((q) => (
              <li key={q.id}>
                <Link to={`/quotes/${q.id}`} className="quote-list__item">
                  <span className="quote-list__service">
                    {q.jobSummary.serviceType}
                  </span>
                  <span className="quote-list__preview">
                    {truncate(q.followUpMessage, 60)}
                  </span>
                  <span className="quote-list__meta">
                    <span className="quote-list__price">
                      ${q.estimatedPrice.toFixed(2)}
                    </span>
                    <span className="quote-list__date">
                      {formatDate(q.createdAt ?? "")}
                    </span>
                  </span>
                </Link>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

import { useState, useEffect } from "react";
import { useParams, Link } from "react-router-dom";
import { getQuote } from "../../application/api";
import type { QuoteDraft } from "../../domain/types";
import "./NewQuotePage.css";

export function QuoteDetailPage(): React.JSX.Element {
  const { id } = useParams<{ id: string }>();
  const [quote, setQuote] = useState<QuoteDraft | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!id) return;
    getQuote(id)
      .then(setQuote)
      .catch(() => setError("Failed to load quote. Please try again."))
      .finally(() => setLoading(false));
  }, [id]);

  function handleCopy(): void {
    if (!quote) return;
    navigator.clipboard.writeText(quote.followUpMessage).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    });
  }

  if (loading) {
    return (
      <div className="quote-page">
        <p className="quote-page__loading">Loading...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div className="quote-page">
        <div className="error-banner" role="alert">{error}</div>
        <Link to="/" className="quote-page__back">Back to dashboard</Link>
      </div>
    );
  }

  if (!quote) {
    return (
      <div className="quote-page">
        <p className="quote-page__empty">Quote not found.</p>
        <Link to="/" className="quote-page__back">Back to dashboard</Link>
      </div>
    );
  }

  return (
    <div className="quote-page">
      <Link to="/" className="quote-page__back">Back to dashboard</Link>

      <section className="result-section">
        <h3 className="result-heading">Job Summary</h3>
        <div className="result-grid">
          <div className="result-item">
            <span className="result-label">Service</span>
            <span className="result-value">{quote.jobSummary.serviceType}</span>
          </div>
          {quote.jobSummary.propertySize && (
            <div className="result-item">
              <span className="result-label">Property</span>
              <span className="result-value">{quote.jobSummary.propertySize}</span>
            </div>
          )}
          {quote.jobSummary.requestedDate && (
            <div className="result-item">
              <span className="result-label">Date</span>
              <span className="result-value">{quote.jobSummary.requestedDate}</span>
            </div>
          )}
          {Object.entries(quote.jobSummary.extractedDetails).map(([key, value]) => (
            <div className="result-item" key={key}>
              <span className="result-label">{key}</span>
              <span className="result-value">{value}</span>
            </div>
          ))}
        </div>
      </section>

      <section className="result-section">
        <h3 className="result-heading">Price Breakdown</h3>
        <div className="price-breakdown">
          {quote.priceBreakdown.map((item, i) => (
            <div className="price-line" key={i}>
              <span>{item.description}</span>
              <span className="price-amount">${item.amount.toFixed(2)}</span>
            </div>
          ))}
          <div className="price-total">
            <span>Total</span>
            <span className="price-amount">${quote.estimatedPrice.toFixed(2)}</span>
          </div>
        </div>
      </section>

      <section className="result-section">
        <div className="message-header">
          <h3 className="result-heading">Follow-up Message</h3>
          <button type="button" className="btn-copy" onClick={handleCopy}>
            {copied ? "Copied!" : "Copy"}
          </button>
        </div>
        <div className="message-preview">{quote.followUpMessage}</div>
      </section>

      {quote.clarificationMessage && (
        <section className="result-section">
          <h3 className="result-heading">Clarification Needed</h3>
          <div className="message-preview message-preview--warning">
            {quote.clarificationMessage}
          </div>
        </section>
      )}
    </div>
  );
}

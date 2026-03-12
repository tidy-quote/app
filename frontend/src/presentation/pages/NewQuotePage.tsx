import { useState, type FormEvent, type ChangeEvent } from "react";
import type { QuoteDraft, ToneOption } from "../../domain/types";
import { generateQuote } from "../../application/api";
import { fileToBase64 } from "../../application/file-utils";
import "./NewQuotePage.css";

const TONE_OPTIONS: { value: ToneOption; label: string }[] = [
  { value: "friendly", label: "Friendly" },
  { value: "direct", label: "Direct" },
  { value: "premium", label: "Premium" },
];

export function NewQuotePage(): React.JSX.Element {
  const [rawText, setRawText] = useState("");
  const [files, setFiles] = useState<File[]>([]);
  const [tone, setTone] = useState<ToneOption>("friendly");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [result, setResult] = useState<QuoteDraft | null>(null);
  const [copied, setCopied] = useState(false);

  function handleFileChange(e: ChangeEvent<HTMLInputElement>): void {
    if (e.target.files) {
      setFiles(Array.from(e.target.files));
    }
  }

  async function handleSubmit(e: FormEvent): Promise<void> {
    e.preventDefault();
    setLoading(true);
    setError("");

    try {
      const imageDataUrls = await Promise.all(files.map(fileToBase64));
      const quote = await generateQuote(rawText, imageDataUrls, tone);
      setResult(quote);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to generate quote");
    } finally {
      setLoading(false);
    }
  }

  function handleReset(): void {
    setRawText("");
    setFiles([]);
    setTone("friendly");
    setResult(null);
    setError("");
    setCopied(false);
  }

  async function handleCopy(): Promise<void> {
    if (!result) return;
    await navigator.clipboard.writeText(result.followUpMessage);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  const hasInput = rawText.trim().length > 0 || files.length > 0;

  if (result) {
    return (
      <div className="new-quote">
        <h2 className="page-title">Quote Generated</h2>

        <div className="result-sections">
          <section className="result-card">
            <h3 className="result-card__header">Job Summary</h3>
            <dl className="detail-list">
              <dt>Service Type</dt>
              <dd>{result.jobSummary.serviceType}</dd>
              {result.jobSummary.propertySize && (
                <>
                  <dt>Property Size</dt>
                  <dd>{result.jobSummary.propertySize}</dd>
                </>
              )}
              {result.jobSummary.requestedDate && (
                <>
                  <dt>Date</dt>
                  <dd>{result.jobSummary.requestedDate}</dd>
                </>
              )}
              {result.jobSummary.requestedTime && (
                <>
                  <dt>Time</dt>
                  <dd>{result.jobSummary.requestedTime}</dd>
                </>
              )}
              {Object.entries(result.jobSummary.extractedDetails).map(
                ([key, value]) => (
                  <div key={key} className="detail-entry">
                    <dt>{key}</dt>
                    <dd>{value}</dd>
                  </div>
                )
              )}
            </dl>
          </section>

          {result.jobSummary.missingInfo.length > 0 && (
            <section className="result-card result-card--warning">
              <h3 className="result-card__header">Missing Information</h3>
              <ul className="info-list">
                {result.jobSummary.missingInfo.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
              {result.clarificationMessage && (
                <p className="clarification-text">
                  {result.clarificationMessage}
                </p>
              )}
            </section>
          )}

          <section className="result-card">
            <h3 className="result-card__header">Price Breakdown</h3>
            <table className="price-table">
              <tbody>
                {result.priceBreakdown.map((item) => (
                  <tr key={item.description}>
                    <td>{item.description}</td>
                    <td className="price-amount">
                      {formatCurrency(item.amount)}
                    </td>
                  </tr>
                ))}
              </tbody>
              <tfoot>
                <tr className="price-total-row">
                  <td>Total</td>
                  <td className="price-amount">
                    {formatCurrency(result.estimatedPrice)}
                  </td>
                </tr>
              </tfoot>
            </table>
          </section>

          {result.assumptions.length > 0 && (
            <section className="result-card">
              <h3 className="result-card__header">Assumptions</h3>
              <ul className="info-list">
                {result.assumptions.map((item) => (
                  <li key={item}>{item}</li>
                ))}
              </ul>
            </section>
          )}

          <section className="result-card">
            <h3 className="result-card__header">Follow-Up Message</h3>
            <textarea
              className="followup-textarea"
              readOnly
              value={result.followUpMessage}
              rows={10}
            />
            <button
              type="button"
              className="btn-copy"
              onClick={handleCopy}
            >
              {copied ? "Copied!" : "Copy Message"}
            </button>
          </section>
        </div>

        <button type="button" className="btn-primary btn-reset" onClick={handleReset}>
          New Quote
        </button>
      </div>
    );
  }

  return (
    <div className="new-quote">
      <h2 className="page-title">New Quote</h2>
      <p className="page-desc">
        Paste the customer message or upload photos of the enquiry.
      </p>

      {error && (
        <div className="error-banner" role="alert">
          {error}
        </div>
      )}

      <form onSubmit={handleSubmit} className="quote-form">
        <label className="form-label" htmlFor="lead-text">
          Customer message
        </label>
        <textarea
          id="lead-text"
          className="form-textarea"
          rows={6}
          placeholder="Paste the lead message here..."
          value={rawText}
          onChange={(e) => setRawText(e.target.value)}
          disabled={loading}
        />

        <label className="form-label" htmlFor="lead-photos">
          Photos (optional)
        </label>
        <input
          id="lead-photos"
          type="file"
          accept="image/*"
          multiple
          className="form-file"
          onChange={handleFileChange}
          disabled={loading}
        />
        {files.length > 0 && (
          <p className="file-count">
            {files.length} file{files.length !== 1 ? "s" : ""} selected
          </p>
        )}

        <fieldset className="tone-fieldset">
          <legend className="form-label">Tone</legend>
          <div className="tone-options">
            {TONE_OPTIONS.map((option) => (
              <label key={option.value} className="tone-option">
                <input
                  type="radio"
                  name="tone"
                  value={option.value}
                  checked={tone === option.value}
                  onChange={() => setTone(option.value)}
                  disabled={loading}
                />
                <span className="tone-label">{option.label}</span>
              </label>
            ))}
          </div>
        </fieldset>

        <button
          type="submit"
          className="btn-primary"
          disabled={!hasInput || loading}
        >
          {loading ? "Generating..." : "Generate Quote"}
        </button>
      </form>
    </div>
  );
}

function formatCurrency(amount: number): string {
  return `\u00A3${amount.toFixed(2)}`;
}

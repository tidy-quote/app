import { useState, type FormEvent, type ChangeEvent } from "react";
import "./NewQuotePage.css";

export function NewQuotePage(): React.JSX.Element {
  const [rawText, setRawText] = useState("");
  const [files, setFiles] = useState<File[]>([]);

  function handleFileChange(e: ChangeEvent<HTMLInputElement>): void {
    if (e.target.files) {
      setFiles(Array.from(e.target.files));
    }
  }

  function handleSubmit(e: FormEvent): void {
    e.preventDefault();
    // TODO: wire up to API
    console.log("Submitting lead:", { rawText, fileCount: files.length });
  }

  const hasInput = rawText.trim().length > 0 || files.length > 0;

  return (
    <div className="new-quote">
      <h2 className="page-title">New Quote</h2>
      <p className="page-desc">
        Paste the customer message or upload photos of the enquiry.
      </p>

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
        />
        {files.length > 0 && (
          <p className="file-count">
            {files.length} file{files.length !== 1 ? "s" : ""} selected
          </p>
        )}

        <button type="submit" className="btn-primary" disabled={!hasInput}>
          Generate Quote
        </button>
      </form>
    </div>
  );
}

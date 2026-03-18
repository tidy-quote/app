import { useState, useEffect, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { getSubscriptionStatus } from "../../application/api";
import "./AuthPages.css";

const MAX_POLLS = 20;
const POLL_INTERVAL_MS = 3000;

export function CheckoutSuccessPage(): React.JSX.Element {
  const navigate = useNavigate();
  const [status, setStatus] = useState<"polling" | "active" | "timeout">("polling");
  const pollCount = useRef(0);

  useEffect(() => {
    const interval = setInterval(async () => {
      pollCount.current += 1;

      try {
        const sub = await getSubscriptionStatus();
        if (sub.status === "active") {
          setStatus("active");
          clearInterval(interval);
          setTimeout(() => navigate("/"), 1500);
        }
      } catch {
        // Keep polling on error
      }

      if (pollCount.current >= MAX_POLLS) {
        setStatus("timeout");
        clearInterval(interval);
      }
    }, POLL_INTERVAL_MS);

    return () => clearInterval(interval);
  }, [navigate]);

  return (
    <div className="auth-page">
      <div className="auth-card">
        <h1 className="auth-logo">Tidy-Quote</h1>

        <div role="status" aria-live="polite">
          {status === "polling" && (
            <>
              <h2 className="auth-title">Payment received</h2>
              <p className="auth-message">Activating your subscription...</p>
            </>
          )}

          {status === "active" && (
            <>
              <h2 className="auth-title">Subscription active</h2>
              <p className="auth-message">Redirecting to the app...</p>
            </>
          )}

          {status === "timeout" && (
            <>
              <h2 className="auth-title">Almost there</h2>
              <p className="auth-message">
                Your payment was received but activation is taking longer than expected.
                Please refresh in a moment or contact support.
              </p>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

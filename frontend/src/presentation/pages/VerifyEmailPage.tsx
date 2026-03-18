import { useState, useEffect } from "react";
import { useSearchParams, useNavigate, Link } from "react-router-dom";
import { verifyEmail, resendVerification } from "../../application/api";
import "./AuthPages.css";

export function VerifyEmailPage(): React.JSX.Element {
  const [searchParams] = useSearchParams();
  const token = searchParams.get("token");
  const navigate = useNavigate();

  const [status, setStatus] = useState<"pending" | "verifying" | "verified" | "error">(
    token ? "verifying" : "pending"
  );
  const [error, setError] = useState("");
  const [resending, setResending] = useState(false);
  const [resent, setResent] = useState(false);

  useEffect(() => {
    if (!token) return;

    verifyEmail(token)
      .then(() => {
        setStatus("verified");
        setTimeout(() => navigate("/"), 2000);
      })
      .catch((err) => {
        setStatus("error");
        setError(err instanceof Error ? err.message : "Verification failed");
      });
  }, [token, navigate]);

  async function handleResend(): Promise<void> {
    setResending(true);
    setResent(false);

    try {
      await resendVerification();
      setResent(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to resend");
    } finally {
      setResending(false);
    }
  }

  return (
    <div className="auth-page">
      <div className="auth-card">
        <h1 className="auth-logo">Tidy-Quote</h1>

        {status === "verifying" && (
          <>
            <h2 className="auth-title">Verifying your email...</h2>
            <p className="auth-message">Please wait.</p>
          </>
        )}

        {status === "verified" && (
          <>
            <h2 className="auth-title">Email verified</h2>
            <p className="auth-message">Redirecting to the app...</p>
          </>
        )}

        {status === "pending" && (
          <>
            <h2 className="auth-title">Check your email</h2>
            <p className="auth-message">
              We sent a verification link to your email address. Click it to activate your account.
            </p>

            {resent && (
              <div className="success-banner" role="status">
                Verification email sent
              </div>
            )}

            <button
              type="button"
              className="btn-primary auth-submit"
              onClick={handleResend}
              disabled={resending}
            >
              {resending ? "Sending..." : "Resend verification email"}
            </button>

            <p className="auth-switch">
              <Link to="/login" className="auth-link">
                Back to login
              </Link>
            </p>
          </>
        )}

        {status === "error" && (
          <>
            <h2 className="auth-title">Verification failed</h2>
            {error && (
              <div className="error-banner" role="alert">
                {error}
              </div>
            )}

            <button
              type="button"
              className="btn-primary auth-submit"
              onClick={handleResend}
              disabled={resending}
            >
              {resending ? "Sending..." : "Resend verification email"}
            </button>

            <p className="auth-switch">
              <Link to="/login" className="auth-link">
                Back to login
              </Link>
            </p>
          </>
        )}
      </div>
    </div>
  );
}

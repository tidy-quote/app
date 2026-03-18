import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import "./AuthPages.css";

export function CheckoutSuccessPage(): React.JSX.Element {
  const navigate = useNavigate();

  useEffect(() => {
    const timer = setTimeout(() => navigate("/"), 3000);
    return () => clearTimeout(timer);
  }, [navigate]);

  return (
    <div className="auth-page">
      <div className="auth-card">
        <h1 className="auth-logo">Tidy-Quote</h1>
        <h2 className="auth-title">Payment successful</h2>
        <p className="auth-message">
          Your subscription is active. Redirecting to the app...
        </p>
      </div>
    </div>
  );
}

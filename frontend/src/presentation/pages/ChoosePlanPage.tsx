import { useState, useEffect } from "react";
import { createCheckoutSession, getPlans, type PlanInfo } from "../../application/api";
import "./AuthPages.css";

export function ChoosePlanPage(): React.JSX.Element {
  const [plans, setPlans] = useState<PlanInfo[]>([]);
  const [loadingPlans, setLoadingPlans] = useState(true);
  const [loading, setLoading] = useState<string | null>(null);
  const [error, setError] = useState("");

  useEffect(() => {
    getPlans()
      .then(setPlans)
      .catch(() => setError("Failed to load plans"))
      .finally(() => setLoadingPlans(false));
  }, []);

  async function handleChoose(priceId: string): Promise<void> {
    setError("");
    setLoading(priceId);

    try {
      const url = await createCheckoutSession(priceId);
      window.location.assign(url);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Checkout failed");
      setLoading(null);
    }
  }

  return (
    <div className="auth-page">
      <div className="plans-container">
        <h1 className="auth-logo">Tidy-Quote</h1>
        <h2 className="auth-title">Choose your plan</h2>

        {error && (
          <div className="error-banner" role="alert">
            {error}
          </div>
        )}

        {loadingPlans && (
          <p className="auth-message">Loading plans...</p>
        )}

        {!loadingPlans && plans.length > 0 && (
          <div className="plans-grid">
            {plans.map((plan) => (
              <div
                key={plan.priceId}
                className={`plan-card${plan.featured ? " plan-card--featured" : ""}`}
              >
                {plan.featured && <span className="plan-badge">Most Popular</span>}
                <h3 className="plan-name">{plan.name}</h3>
                <div className="plan-price">
                  {plan.price}
                  <span>/mo</span>
                </div>
                <p className="plan-desc">{plan.description}</p>
                <ul className="plan-features">
                  {plan.features.map((f) => (
                    <li key={f}>{f}</li>
                  ))}
                </ul>
                <button
                  type="button"
                  className={`btn-primary plan-cta${plan.featured ? "" : " plan-cta--outline"}`}
                  onClick={() => handleChoose(plan.priceId)}
                  disabled={loading !== null}
                >
                  {loading === plan.priceId ? "Redirecting..." : "Get Started"}
                </button>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

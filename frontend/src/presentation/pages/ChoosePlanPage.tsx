import { useState } from "react";
import { createCheckoutSession } from "../../application/api";
import "./AuthPages.css";

interface Plan {
  name: string;
  priceId: string;
  price: string;
  description: string;
  features: string[];
  featured?: boolean;
}

const PRICE_STARTER = import.meta.env.VITE_STRIPE_PRICE_STARTER ?? "";
const PRICE_SOLO = import.meta.env.VITE_STRIPE_PRICE_SOLO ?? "";
const PRICE_PRO = import.meta.env.VITE_STRIPE_PRICE_PRO ?? "";

const PLANS: Plan[] = [
  {
    name: "Starter",
    priceId: PRICE_STARTER,
    price: "$1.99",
    description: "Try it out with a few quotes each month.",
    features: [
      "5 AI quote generations per month",
      "1 pricing template",
      "Job summary extraction",
      "Follow-up message drafts",
    ],
  },
  {
    name: "Solo",
    priceId: PRICE_SOLO,
    price: "$8.99",
    description: "For cleaners quoting multiple jobs a week.",
    features: [
      "75 AI quote generations per month",
      "Multiple pricing templates",
      "All tone options",
      "Photo & screenshot uploads",
    ],
    featured: true,
  },
  {
    name: "Pro",
    priceId: PRICE_PRO,
    price: "$19.99",
    description: "For busy cleaners who quote every day.",
    features: [
      "Unlimited quote generations",
      "Multi-location pricing templates",
      "Priority AI processing",
      "Everything in Solo",
    ],
  },
];

export function ChoosePlanPage(): React.JSX.Element {
  const [loading, setLoading] = useState<string | null>(null);
  const [error, setError] = useState("");

  async function handleChoose(priceId: string): Promise<void> {
    setError("");
    setLoading(priceId);

    try {
      const url = await createCheckoutSession(priceId);
      window.location.href = url;
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

        <div className="plans-grid">
          {PLANS.map((plan) => (
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
      </div>
    </div>
  );
}

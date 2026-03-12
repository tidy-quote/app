import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import { getPricingTemplate } from "../../application/api";
import "./DashboardPage.css";

export function DashboardPage(): React.JSX.Element {
  const [hasPricing, setHasPricing] = useState<boolean | null>(null);

  useEffect(() => {
    getPricingTemplate()
      .then((template) => setHasPricing(template !== null))
      .catch(() => setHasPricing(false));
  }, []);

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

      <div className="quick-actions">
        <Link to="/quote/new" className="action-card action-card--primary">
          <span className="action-card__title">New Quote</span>
          <span className="action-card__desc">
            Paste a lead message or upload photos to generate a quote
          </span>
        </Link>

        <Link to="/pricing" className="action-card">
          <span className="action-card__title">Pricing Setup</span>
          <span className="action-card__desc">
            {hasPricing
              ? "View or update your service categories, add-ons, and rates"
              : "Configure your service categories, add-ons, and rates"}
          </span>
        </Link>
      </div>
    </div>
  );
}

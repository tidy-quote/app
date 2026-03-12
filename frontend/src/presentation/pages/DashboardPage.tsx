import { Link } from "react-router-dom";
import "./DashboardPage.css";

export function DashboardPage(): React.JSX.Element {
  return (
    <div className="dashboard">
      <h2 className="dashboard-title">Welcome to QuoteSnap</h2>
      <p className="dashboard-subtitle">
        Generate professional quotes from customer messages in seconds.
      </p>

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
            Configure your service categories, add-ons, and rates
          </span>
        </Link>
      </div>
    </div>
  );
}

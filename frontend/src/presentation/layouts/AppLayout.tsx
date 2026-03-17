import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { useAuth } from "../components/useAuth";
import "./AppLayout.css";

function DashboardIcon(): React.JSX.Element {
  return (
    <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
      <polyline points="9 22 9 12 15 12 15 22" />
    </svg>
  );
}

function QuoteIcon(): React.JSX.Element {
  return (
    <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
      <polyline points="14 2 14 8 20 8" />
      <line x1="16" y1="13" x2="8" y2="13" />
      <line x1="16" y1="17" x2="8" y2="17" />
    </svg>
  );
}

function PricingIcon(): React.JSX.Element {
  return (
    <svg className="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      <line x1="12" y1="1" x2="12" y2="23" />
      <path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6" />
    </svg>
  );
}

export function AppLayout(): React.JSX.Element {
  const { logout } = useAuth();
  const navigate = useNavigate();

  function handleLogout(): void {
    logout();
    navigate("/login");
  }

  return (
    <div className="app-layout">
      <header className="app-header">
        <h1 className="app-logo"><img src="/logo.svg" alt="" className="app-logo-img" />Tidy-Quote</h1>
        <button type="button" className="btn-logout" onClick={handleLogout}>
          Log out
        </button>
      </header>

      <main className="app-main">
        <Outlet />
      </main>

      <nav className="app-nav">
        <NavLink to="/" end className="nav-link">
          <DashboardIcon />
          Dashboard
        </NavLink>
        <NavLink to="/quote/new" className="nav-link">
          <QuoteIcon />
          New Quote
        </NavLink>
        <NavLink to="/pricing" className="nav-link">
          <PricingIcon />
          Pricing
        </NavLink>
      </nav>
    </div>
  );
}

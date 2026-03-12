import { NavLink, Outlet } from "react-router-dom";
import "./AppLayout.css";

export function AppLayout(): React.JSX.Element {
  return (
    <div className="app-layout">
      <header className="app-header">
        <h1 className="app-logo">QuoteSnap</h1>
      </header>

      <main className="app-main">
        <Outlet />
      </main>

      <nav className="app-nav">
        <NavLink to="/" end className="nav-link">
          Dashboard
        </NavLink>
        <NavLink to="/quote/new" className="nav-link">
          New Quote
        </NavLink>
        <NavLink to="/pricing" className="nav-link">
          Pricing
        </NavLink>
      </nav>
    </div>
  );
}

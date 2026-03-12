import { NavLink, Outlet, useNavigate } from "react-router-dom";
import { useAuth } from "../components/AuthProvider";
import "./AppLayout.css";

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
        <h1 className="app-logo">QuoteSnap</h1>
        <button type="button" className="btn-logout" onClick={handleLogout}>
          Log out
        </button>
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

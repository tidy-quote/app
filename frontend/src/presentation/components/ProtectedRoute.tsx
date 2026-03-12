import { Navigate, Outlet } from "react-router-dom";
import { useAuth } from "./useAuth";

export function ProtectedRoute(): React.JSX.Element {
  const { isAuthenticated } = useAuth();

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  return <Outlet />;
}

import { Navigate, Outlet } from "react-router-dom";
import { useAuth } from "./useAuth";

export function SubscriptionRoute(): React.JSX.Element {
  const { subscriptionStatus } = useAuth();

  if (subscriptionStatus === "unknown") {
    return <div className="page-loading">Loading...</div>;
  }

  if (subscriptionStatus !== "active") {
    return <Navigate to="/choose-plan" replace />;
  }

  return <Outlet />;
}

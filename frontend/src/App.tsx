import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AuthProvider } from "./presentation/components/AuthProvider";
import { ProtectedRoute } from "./presentation/components/ProtectedRoute";
import { AppLayout } from "./presentation/layouts/AppLayout";
import { DashboardPage } from "./presentation/pages/DashboardPage";
import { LoginPage } from "./presentation/pages/LoginPage";
import { NewQuotePage } from "./presentation/pages/NewQuotePage";
import { PricingSetupPage } from "./presentation/pages/PricingSetupPage";
import { SignupPage } from "./presentation/pages/SignupPage";

export function App(): React.JSX.Element {
  return (
    <BrowserRouter>
      <AuthProvider>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route path="/signup" element={<SignupPage />} />
          <Route element={<ProtectedRoute />}>
            <Route element={<AppLayout />}>
              <Route path="/" element={<DashboardPage />} />
              <Route path="/quote/new" element={<NewQuotePage />} />
              <Route path="/pricing" element={<PricingSetupPage />} />
            </Route>
          </Route>
        </Routes>
      </AuthProvider>
    </BrowserRouter>
  );
}

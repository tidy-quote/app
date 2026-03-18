import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AuthProvider } from "./presentation/components/AuthProvider";
import { ProtectedRoute } from "./presentation/components/ProtectedRoute";
import { AppLayout } from "./presentation/layouts/AppLayout";
import { DashboardPage } from "./presentation/pages/DashboardPage";
import { ForgotPasswordPage } from "./presentation/pages/ForgotPasswordPage";
import { LoginPage } from "./presentation/pages/LoginPage";
import { NewQuotePage } from "./presentation/pages/NewQuotePage";
import { PricingSetupPage } from "./presentation/pages/PricingSetupPage";
import { ResetPasswordPage } from "./presentation/pages/ResetPasswordPage";
import { SignupPage } from "./presentation/pages/SignupPage";
import { VerifyEmailPage } from "./presentation/pages/VerifyEmailPage";

export function App(): React.JSX.Element {
  return (
    <BrowserRouter>
      <AuthProvider>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route path="/signup" element={<SignupPage />} />
          <Route path="/verify" element={<VerifyEmailPage />} />
          <Route path="/forgot-password" element={<ForgotPasswordPage />} />
          <Route path="/reset-password" element={<ResetPasswordPage />} />
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

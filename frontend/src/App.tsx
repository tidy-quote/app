import { lazy, Suspense } from "react";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AuthProvider } from "./presentation/components/AuthProvider";
import { ProtectedRoute } from "./presentation/components/ProtectedRoute";
import { AppLayout } from "./presentation/layouts/AppLayout";

const LoginPage = lazy(() => import("./presentation/pages/LoginPage").then(m => ({ default: m.LoginPage })));
const SignupPage = lazy(() => import("./presentation/pages/SignupPage").then(m => ({ default: m.SignupPage })));
const VerifyEmailPage = lazy(() => import("./presentation/pages/VerifyEmailPage").then(m => ({ default: m.VerifyEmailPage })));
const ForgotPasswordPage = lazy(() => import("./presentation/pages/ForgotPasswordPage").then(m => ({ default: m.ForgotPasswordPage })));
const ResetPasswordPage = lazy(() => import("./presentation/pages/ResetPasswordPage").then(m => ({ default: m.ResetPasswordPage })));
const ChoosePlanPage = lazy(() => import("./presentation/pages/ChoosePlanPage").then(m => ({ default: m.ChoosePlanPage })));
const CheckoutSuccessPage = lazy(() => import("./presentation/pages/CheckoutSuccessPage").then(m => ({ default: m.CheckoutSuccessPage })));
const DashboardPage = lazy(() => import("./presentation/pages/DashboardPage").then(m => ({ default: m.DashboardPage })));
const NewQuotePage = lazy(() => import("./presentation/pages/NewQuotePage").then(m => ({ default: m.NewQuotePage })));
const QuoteDetailPage = lazy(() => import("./presentation/pages/QuoteDetailPage").then(m => ({ default: m.QuoteDetailPage })));
const PricingSetupPage = lazy(() => import("./presentation/pages/PricingSetupPage").then(m => ({ default: m.PricingSetupPage })));

export function App(): React.JSX.Element {
  return (
    <BrowserRouter>
      <AuthProvider>
        <Suspense fallback={<div className="page-loading">Loading...</div>}>
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/signup" element={<SignupPage />} />
            <Route path="/verify" element={<VerifyEmailPage />} />
            <Route path="/forgot-password" element={<ForgotPasswordPage />} />
            <Route path="/reset-password" element={<ResetPasswordPage />} />
            <Route path="/choose-plan" element={<ChoosePlanPage />} />
            <Route path="/checkout-success" element={<CheckoutSuccessPage />} />
            <Route element={<ProtectedRoute />}>
              <Route element={<AppLayout />}>
                <Route path="/" element={<DashboardPage />} />
                <Route path="/quote/new" element={<NewQuotePage />} />
                <Route path="/quotes/:id" element={<QuoteDetailPage />} />
                <Route path="/pricing" element={<PricingSetupPage />} />
              </Route>
            </Route>
          </Routes>
        </Suspense>
      </AuthProvider>
    </BrowserRouter>
  );
}

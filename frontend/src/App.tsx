import { BrowserRouter, Routes, Route } from "react-router-dom";
import { AppLayout } from "./presentation/layouts/AppLayout";
import { DashboardPage } from "./presentation/pages/DashboardPage";
import { NewQuotePage } from "./presentation/pages/NewQuotePage";
import { PricingSetupPage } from "./presentation/pages/PricingSetupPage";

export function App(): React.JSX.Element {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppLayout />}>
          <Route path="/" element={<DashboardPage />} />
          <Route path="/quote/new" element={<NewQuotePage />} />
          <Route path="/pricing" element={<PricingSetupPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}

import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Route, Routes } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

import "./index.css";

import { Layout } from "./components/Layout";
import { Dashboard } from "./pages/Dashboard";
import { ServerDetail } from "./pages/ServerDetail";
import { SessionView } from "./pages/SessionView";
import { TraceView } from "./pages/TraceView";
import { DiffView } from "./pages/DiffView";
import { ReplayPanel } from "./pages/ReplayPanel";
import { SearchPage } from "./pages/Search";
import { SettingsPage } from "./pages/Settings";
import { ToastProvider, ToastBridge } from "./components/Toast";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
});

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <ToastBridge />
        <BrowserRouter>
          <Routes>
            <Route element={<Layout />}>
              <Route path="/" element={<Dashboard />} />
              <Route path="/servers" element={<Dashboard />} />
              <Route path="/servers/:name" element={<ServerDetail />} />
              <Route path="/sessions/:id" element={<SessionView />} />
              <Route path="/traces/:id" element={<TraceView />} />
              <Route path="/diff" element={<DiffView />} />
              <Route path="/replay" element={<ReplayPanel />} />
              <Route path="/search" element={<SearchPage />} />
              <Route path="/settings" element={<SettingsPage />} />
            </Route>
          </Routes>
        </BrowserRouter>
      </ToastProvider>
    </QueryClientProvider>
  </StrictMode>,
);

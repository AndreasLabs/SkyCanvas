import React from "react";
import { Route, Routes, Navigate } from "react-router-dom";
import { AppLayout } from "./layout/app_layout";
import { Home } from "./pages/home";
import { ArduPilotGCS } from "./pages/ArduPilotGCS";

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route path="/" element={<Home />} />
        <Route path="/ardupilot" element={<ArduPilotGCS />} />
        {/* The wildcard route must be last to properly handle 404 cases */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
} 
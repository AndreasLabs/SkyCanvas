import React from 'react';
import { AppLayout } from '../layout/app_layout';
import { Routes, Route, Navigate } from 'react-router-dom';
import { Home } from '../pages/home';
import { RedisDemo } from '../pages/RedisDemo';

export function AppRoutes() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route path="/" element={<Home />} />
        <Route path="/redis-demo" element={<RedisDemo />} />
        {/* The wildcard route must be last to properly handle 404 cases */}
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
} 
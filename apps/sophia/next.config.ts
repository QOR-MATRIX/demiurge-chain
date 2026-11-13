// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import type { NextConfig } from "next";

const config: NextConfig = {
  reactStrictMode: true,
  // Use webpack mode for Tailwind v3 compatibility
  // turbopack: {},

  /* Skip type checking during build — pre-existing issues in codebase */
  typescript: {
    ignoreBuildErrors: true,
  },
  eslint: {
    ignoreDuringBuilds: true,
  },
  
  /* Environment variables */
  env: {
    NEXT_PUBLIC_DEMIURGE_RPC_URL: process.env.NEXT_PUBLIC_DEMIURGE_RPC_URL || "http://localhost:9944",
    NEXT_PUBLIC_QOR_AUTH_URL: process.env.NEXT_PUBLIC_QOR_AUTH_URL || "http://localhost:3001",
    NEXT_PUBLIC_API_URL: process.env.NEXT_PUBLIC_API_URL || "http://localhost:3003/api",
  },

  /* CORS and headers */
  headers: async () => {
    return [
      {
        source: "/:path*",
        headers: [
          {
            key: "X-DNS-Prefetch-Control",
            value: "on",
          },
          {
            key: "X-Frame-Options",
            value: "DENY",
          },
          {
            key: "X-Content-Type-Options",
            value: "nosniff",
          },
          {
            key: "X-XSS-Protection",
            value: "1; mode=block",
          },
          {
            key: "Referrer-Policy",
            value: "strict-origin-when-cross-origin",
          },
          {
            key: "Content-Security-Policy",
            value: "default-src 'self'; script-src 'self' 'unsafe-inline' 'unsafe-eval'; style-src 'self' 'unsafe-inline'; connect-src 'self' localhost:* http://localhost:* wss://localhost:* https://*.qorsync.dev; img-src 'self' data: https:; font-src 'self' data:;",
          },
        ],
      },
    ];
  },

  /* Image optimization */
  images: {
    remotePatterns: [
      {
        protocol: "https",
        hostname: "**.qorsync.dev",
      },
      {
        protocol: "http",
        hostname: "localhost",
      },
    ],
  },

  /* Webpack configuration for optimizations */
  webpack: (config: any, { isServer }: { isServer: boolean }) => {
    return config;
  },
};

export default config;

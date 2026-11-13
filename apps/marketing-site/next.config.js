// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/** @type {import('next').NextConfig} */
const nextConfig = {
  reactStrictMode: true,
  transpilePackages: ['@demiurge/qor-sdk', '@demiurge/ui-shared'],
  output: 'standalone',
  env: {
    NEXT_PUBLIC_QOR_AUTH_URL: process.env.NEXT_PUBLIC_QOR_AUTH_URL || 'http://127.0.0.1:8080/api/v1',
    NEXT_PUBLIC_RPC_URL: process.env.NEXT_PUBLIC_RPC_URL || 'ws://127.0.0.1:9944',
    NEXT_PUBLIC_SOPHIA_API_URL: process.env.NEXT_PUBLIC_SOPHIA_API_URL || '/api/sophia',
  },
  async headers() {
    return [
      {
        source: '/:path*',
        headers: [
          {
            key: 'X-Content-Type-Options',
            value: 'nosniff',
          },
          {
            key: 'X-Frame-Options',
            value: 'DENY',
          },
          {
            key: 'X-XSS-Protection',
            value: '1; mode=block',
          },
        ],
      },
    ];
  },
};

module.exports = nextConfig;

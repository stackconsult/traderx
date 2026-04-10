/** @type {import('next').NextConfig} */
const nextConfig = {
  // Enable Partial Prerendering for sub-3s load times
  experimental: {
    ppr: true,
    optimizePackageImports: ['shadcn-ui', 'lucide-react']
  },
  
  // App Router configuration
  appDir: true,
  
  // Standalone output for Docker
  output: 'standalone',
  
  // Image optimization
  images: {
    domains: ['localhost'],
    formats: ['image/webp', 'image/avif']
  },
  
  // Performance optimizations
  swcMinify: true,
  poweredByHeader: false,
  
  // Environment variables
  env: {
    NEXT_PUBLIC_APP_URL: process.env.NEXT_PUBLIC_APP_URL || 'http://localhost:3000'
  },
  
  // Webpack configuration for streaming
  webpack: (config) => {
    config.resolve.fallback = {
      ...config.resolve.fallback,
      fs: false,
    };
    return config;
  },
  
  // Headers for security
  async headers() {
    return [
      {
        source: '/(.*)',
        headers: [
          {
            key: 'X-Frame-Options',
            value: 'DENY'
          },
          {
            key: 'X-Content-Type-Options',
            value: 'nosniff'
          }
        ]
      }
    ];
  }
};

module.exports = nextConfig;

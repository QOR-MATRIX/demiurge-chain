// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
'use client';

import { useAuth } from '@/contexts/AuthContext';
import { QorIdAuthFlow } from './QorIdAuthFlow';
import { TerminalButton } from '@/components/terminal';

/**
 * AuthGate Component
 * 
 * Master authentication gate for the Demiurge ecosystem.
 * "The Architect" design - Cyber-Industrial Command Center aesthetic.
 * 
 * SECURITY PRINCIPLE:
 * - Users MUST authenticate with QOR ID BEFORE accessing ANY on-chain features
 * - Unauthenticated users see ONLY the login/signup flow
 * - Once authenticated, users NEVER see login prompts again during their session
 */
export function AuthGate({ children }: { children: React.ReactNode }) {
  const { user, loading, isAuthenticated } = useAuth();

  // ─────────────────────────────────────────────────────────────────────────
  // Loading State - Industrial spinner
  // ─────────────────────────────────────────────────────────────────────────
  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center" style={{ backgroundColor: '#0B0C10' }}>
        <div className="text-center">
          <div className="relative w-16 h-16 mx-auto mb-6">
            {/* Spinning ring - sharp corners */}
            <div 
              className="absolute inset-0 border animate-spin"
              style={{ 
                borderColor: 'rgba(255, 106, 0, 0.3)',
                animationDuration: '3s',
                borderRadius: '2px',
              }} 
            />
            {/* Inner glow */}
            <div 
              className="absolute inset-3 animate-breathing"
              style={{ 
                backgroundColor: 'rgba(255, 106, 0, 0.1)',
                borderRadius: '2px',
              }} 
            />
            {/* Center dot */}
            <div 
              className="absolute inset-[45%]"
              style={{ 
                backgroundColor: '#FF6A00',
                boxShadow: '0 0 12px rgba(255, 106, 0, 0.5)',
                borderRadius: '2px',
              }} 
            />
          </div>
          <p 
            className="text-xs tracking-widest uppercase"
            style={{ 
              fontFamily: "'Rajdhani', sans-serif",
              color: '#7B8794',
              letterSpacing: '2px',
            }}
          >
            INITIALIZING SYSTEM...
          </p>
        </div>
      </div>
    );
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Unauthenticated State - Command Center Login
  // ─────────────────────────────────────────────────────────────────────────
  if (!isAuthenticated) {
    return (
      <div 
        className="h-screen relative overflow-hidden flex flex-col"
        style={{ backgroundColor: '#0B0C10' }}
      >
        {/* Industrial grid background */}
        <div 
          className="absolute inset-0 opacity-[0.03]"
          style={{
            backgroundImage: `
              linear-gradient(rgba(255, 106, 0, 0.5) 1px, transparent 1px),
              linear-gradient(90deg, rgba(255, 106, 0, 0.5) 1px, transparent 1px)
            `,
            backgroundSize: '60px 60px',
          }}
        />
        
        {/* Subtle radial gradient for depth */}
        <div 
          className="absolute inset-0"
          style={{
            background: `
              radial-gradient(ellipse 80% 60% at 50% 20%, rgba(255, 106, 0, 0.04) 0%, transparent 50%),
              radial-gradient(ellipse 60% 40% at 50% 80%, rgba(139, 0, 0, 0.03) 0%, transparent 40%)
            `
          }}
        />
        
        {/* Corner brackets - top left */}
        <div className="absolute top-8 left-8">
          <div className="w-12 h-12" style={{ borderLeft: '1px solid #CC5500', borderTop: '1px solid #CC5500' }} />
        </div>
        {/* Corner brackets - top right */}
        <div className="absolute top-8 right-8">
          <div className="w-12 h-12" style={{ borderRight: '1px solid #CC5500', borderTop: '1px solid #CC5500' }} />
        </div>
        {/* Corner brackets - bottom left */}
        <div className="absolute bottom-8 left-8">
          <div className="w-12 h-12" style={{ borderLeft: '1px solid #CC5500', borderBottom: '1px solid #CC5500' }} />
        </div>
        {/* Corner brackets - bottom right */}
        <div className="absolute bottom-8 right-8">
          <div className="w-12 h-12" style={{ borderRight: '1px solid #CC5500', borderBottom: '1px solid #CC5500' }} />
        </div>

        {/* Main content - vertically centered */}
        <div className="relative z-10 flex-1 flex flex-col items-center justify-center px-4 py-6">
          {/* Logo and branding */}
          <div className="text-center mb-8">
            {/* Logo box - sharp industrial */}
            <div className="relative inline-block group cursor-pointer mb-4">
              <div 
                className="w-16 h-16 flex items-center justify-center mx-auto transition-all duration-300"
                style={{ 
                  backgroundColor: '#1F2833',
                  border: '1px solid #333333',
                  borderRadius: '2px',
                }}
              >
                {/* Corner accents */}
                <div className="absolute -top-px -left-px w-4 h-4" style={{ borderLeft: '2px solid #FF6A00', borderTop: '2px solid #FF6A00' }} />
                <div className="absolute -bottom-px -right-px w-4 h-4" style={{ borderRight: '2px solid #FF6A00', borderBottom: '2px solid #FF6A00' }} />
                <span 
                  className="text-3xl font-semibold"
                  style={{ 
                    fontFamily: "'Rajdhani', sans-serif",
                    color: '#FF6A00',
                    letterSpacing: '2px',
                  }}
                >
                  D
                </span>
              </div>
            </div>
            
            <h1 
              className="text-4xl md:text-5xl mb-2"
              style={{ 
                fontFamily: "'Rajdhani', sans-serif",
                fontWeight: 700,
                letterSpacing: '4px',
                color: '#FFFFFF',
              }}
            >
              DEMIURGE
            </h1>
            <p 
              className="text-xs tracking-widest uppercase"
              style={{ 
                fontFamily: "'JetBrains Mono', monospace",
                color: '#7B8794',
                letterSpacing: '3px',
              }}
            >
              COMMAND TERMINAL v2.0
            </p>
          </div>

          {/* Auth Flow Container */}
          <div className="w-full max-w-md relative">
            {/* Panel with industrial border */}
            <div 
              className="relative p-1"
              style={{
                background: 'linear-gradient(135deg, rgba(255, 106, 0, 0.2) 0%, transparent 50%, rgba(255, 106, 0, 0.1) 100%)',
                borderRadius: '2px',
              }}
            >
              <div 
                style={{ 
                  backgroundColor: '#1F2833',
                  borderRadius: '2px',
                }}
              >
                <QorIdAuthFlow
                  isOpen={true}
                  onClose={() => {}}
                  onSuccess={() => {}}
                  variant="page"
                />
              </div>
            </div>
          </div>

          {/* Status indicators */}
          <div className="mt-8 flex items-center gap-6">
            <div className="flex items-center gap-2">
              <div 
                className="w-2 h-2"
                style={{ 
                  backgroundColor: '#22C55E',
                  boxShadow: '0 0 8px #22C55E',
                  borderRadius: '2px',
                }}
              />
              <span 
                className="text-xs uppercase"
                style={{ 
                  fontFamily: "'Rajdhani', sans-serif",
                  color: '#7B8794',
                  letterSpacing: '1px',
                }}
              >
                SYSTEM ONLINE
              </span>
            </div>
            <div 
              className="h-4 w-px"
              style={{ backgroundColor: '#333333' }}
            />
            <span 
              className="text-xs"
              style={{ 
                fontFamily: "'JetBrains Mono', monospace",
                color: '#4A5568',
                letterSpacing: '1px',
              }}
            >
              Ed25519 · WASM · SECURE
            </span>
          </div>

          {/* Action buttons */}
          <div className="mt-6 flex items-center gap-4">
            {/* Terminal button */}
            <TerminalButton variant="inline" />
            
            {/* Learn more link */}
            <a
              href="https://qorsync.dev"
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center gap-2 px-4 py-2 transition-all duration-200"
              style={{ 
                fontFamily: "'Rajdhani', sans-serif",
                fontSize: '12px',
                fontWeight: 600,
                letterSpacing: '1.5px',
                color: '#FF6A00',
                backgroundColor: 'transparent',
                border: '1px solid #CC5500',
                borderRadius: '0',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = '#FF6A00';
                e.currentTarget.style.color = '#0B0C10';
                e.currentTarget.style.borderColor = '#FF6A00';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent';
                e.currentTarget.style.color = '#FF6A00';
                e.currentTarget.style.borderColor = '#CC5500';
              }}
            >
              <span>LEARN MORE</span>
              <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="square" strokeLinejoin="miter" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
              </svg>
            </a>
          </div>
        </div>
        
        {/* Bottom status bar */}
        <div 
          className="absolute bottom-0 left-0 right-0 h-8 flex items-center justify-center gap-8"
          style={{ 
            backgroundColor: 'rgba(31, 40, 51, 0.8)',
            borderTop: '1px solid #333333',
          }}
        >
          <span 
            className="text-xs"
            style={{ 
              fontFamily: "'JetBrains Mono', monospace",
              color: '#4A5568',
              letterSpacing: '1px',
            }}
          >
            v2.0.0
          </span>
          <span 
            className="text-xs"
            style={{ 
              fontFamily: "'JetBrains Mono', monospace",
              color: '#4A5568',
              letterSpacing: '1px',
            }}
          >
            MAINNET
          </span>
          <span 
            className="text-xs"
            style={{ 
              fontFamily: "'JetBrains Mono', monospace",
              color: '#4A5568',
              letterSpacing: '1px',
            }}
          >
            © 2026 DEMIURGE
          </span>
        </div>
      </div>
    );
  }

  // ─────────────────────────────────────────────────────────────────────────
  // Authenticated - Render application content
  // ─────────────────────────────────────────────────────────────────────────
  return <>{children}</>;
}

// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Spline User Data Webhook
 * 
 * Provides authenticated user-specific data for Spline scenes.
 * Requires Bearer token authentication.
 * 
 * Setup in Spline:
 * 1. Open Variables & Data Panel → APIs tab → New API
 * 2. Set URL: http://127.0.0.1:3000/api/spline/user
 * 3. Method: GET
 * 4. Headers: Add "Authorization: Bearer <token>" (use Spline variables)
 * 5. Map response to user-related Spline variables
 */

import { NextRequest, NextResponse } from 'next/server';

const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://127.0.0.1:9944';

// Simple JWT decode (no verification - for Spline visualization only)
function decodeJWT(token: string): { user_id: string; qor_id: string; role: string; address?: string } | null {
  try {
    const parts = token.split('.');
    if (parts.length !== 3) return null;
    const payload = JSON.parse(Buffer.from(parts[1], 'base64url').toString('utf8'));
    return {
      user_id: payload.user_id || payload.sub || '',
      qor_id: payload.qor_id || payload.sub || '',
      role: payload.role || 'user',
      address: payload.address,
    };
  } catch {
    return null;
  }
}

interface UserData {
  // Identity
  qorId: string;
  address: string;
  isAuthenticated: boolean;
  
  // Balances
  cgtBalance: number;
  cgtBalanceFormatted: string;
  
  // Energy System
  energy: number;
  maxEnergy: number;
  energyPercent: number;
  
  // NFT Stats
  nftCount: number;
  avatarUrl: string | null;
  
  // Level System
  level: number;
  xp: number;
  xpToNextLevel: number;
  levelProgress: number; // 0-1 for Spline animations
  
  // Timestamps
  timestamp: number;
}

async function fetchUserBalance(address: string): Promise<string> {
  try {
    const response = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'chain_getBalance',
        params: [address],
        id: 1,
      }),
      cache: 'no-store',
    });
    
    const data = await response.json();
    return data.result?.balance || '0';
  } catch {
    return '0';
  }
}

async function fetchUserNFTCount(address: string): Promise<number> {
  try {
    const response = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'drc369_getTokensByOwner',
        params: [address],
        id: 1,
      }),
      cache: 'no-store',
    });
    
    const data = await response.json();
    return data.result?.tokens?.length || 0;
  } catch {
    return 0;
  }
}

export async function GET(request: NextRequest) {
  // Check authorization
  const authHeader = request.headers.get('authorization');
  
  if (!authHeader?.startsWith('Bearer ')) {
    return NextResponse.json({
      isAuthenticated: false,
      qorId: 'Guest',
      address: '',
      cgtBalance: 0,
      cgtBalanceFormatted: '0.00 CGT',
      energy: 0,
      maxEnergy: 100,
      energyPercent: 0,
      nftCount: 0,
      avatarUrl: null,
      level: 1,
      xp: 0,
      xpToNextLevel: 100,
      levelProgress: 0,
      timestamp: Date.now(),
    } satisfies UserData, {
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Cache-Control': 'no-cache',
      },
    });
  }

  try {
    const token = authHeader.slice(7);
    const decoded = decodeJWT(token);
    
    if (!decoded) {
      throw new Error('Invalid token');
    }

    const address = decoded.address || '';
    
    // Fetch user data in parallel
    const [balanceRaw, nftCount] = await Promise.all([
      address ? fetchUserBalance(address) : '0',
      address ? fetchUserNFTCount(address) : 0,
    ]);

    // Parse balance (assuming 18 decimals)
    const balanceNum = Number(balanceRaw) / 1e18;
    
    // Calculate level from XP (placeholder - hook to real system)
    const xp = nftCount * 100 + Math.floor(balanceNum / 10);
    const level = Math.floor(Math.sqrt(xp / 100)) + 1;
    const xpForCurrentLevel = Math.pow(level - 1, 2) * 100;
    const xpForNextLevel = Math.pow(level, 2) * 100;
    const xpToNextLevel = xpForNextLevel - xpForCurrentLevel;
    const xpProgress = xp - xpForCurrentLevel;
    const levelProgress = xpProgress / xpToNextLevel;

    const userData: UserData = {
      qorId: decoded.qor_id,
      address: address,
      isAuthenticated: true,
      cgtBalance: parseFloat(balanceNum.toFixed(4)),
      cgtBalanceFormatted: `${balanceNum.toFixed(2)} CGT`,
      energy: 100, // TODO: Hook to energy system
      maxEnergy: 100,
      energyPercent: 1.0,
      nftCount: nftCount,
      avatarUrl: null, // TODO: Fetch from profile
      level: level,
      xp: xp,
      xpToNextLevel: xpToNextLevel,
      levelProgress: Math.min(1, Math.max(0, levelProgress)),
      timestamp: Date.now(),
    };

    return NextResponse.json(userData, {
      headers: {
        'Access-Control-Allow-Origin': '*',
        'Cache-Control': 'private, max-age=5',
      },
    });
  } catch (error) {
    console.error('[Spline User API] Auth error:', error);
    
    return NextResponse.json({
      isAuthenticated: false,
      qorId: 'Guest',
      address: '',
      cgtBalance: 0,
      cgtBalanceFormatted: '0.00 CGT',
      energy: 0,
      maxEnergy: 100,
      energyPercent: 0,
      nftCount: 0,
      avatarUrl: null,
      level: 1,
      xp: 0,
      xpToNextLevel: 100,
      levelProgress: 0,
      timestamp: Date.now(),
    } satisfies UserData, {
      status: 401,
      headers: {
        'Access-Control-Allow-Origin': '*',
      },
    });
  }
}

export async function OPTIONS() {
  return new NextResponse(null, {
    status: 204,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    },
  });
}

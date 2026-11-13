// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Spline Blockchain Data Webhook
 * 
 * This endpoint provides real-time blockchain data that can be consumed by Spline scenes
 * via the Real-time API feature in Spline.
 * 
 * Setup in Spline:
 * 1. Open Variables & Data Panel
 * 2. Click APIs tab → New API
 * 3. Set URL: http://127.0.0.1:3000/api/spline/data
 * 4. Method: GET
 * 5. Enable "Trigger on Load" for auto-updates
 * 6. Map response fields to your Spline variables
 * 
 * Response format matches Spline's expected structure for variable binding.
 */

import { NextRequest, NextResponse } from 'next/server';

// RPC endpoint
const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://127.0.0.1:9944';

interface BlockchainData {
  // Chain State
  blockHeight: number;
  blockTime: number;
  validators: number;
  isConnected: boolean;
  tps: number;
  
  // Network Metrics
  networkLoad: number;      // 0-100 percentage
  peerCount: number;
  finality: number;         // milliseconds
  
  // Computed Animation Values (0-1 range for Spline)
  blockProgress: number;    // Progress to next block
  pulseIntensity: number;   // For glowing effects
  wavePhase: number;        // For wave animations
  
  // Timestamp
  timestamp: number;
  lastUpdate: string;
}

// Cache for rate limiting
let cachedData: BlockchainData | null = null;
let lastFetchTime = 0;
const CACHE_DURATION_MS = 1000; // 1 second cache

async function fetchBlockchainData(): Promise<BlockchainData> {
  const now = Date.now();
  
  // Return cached data if fresh
  if (cachedData && (now - lastFetchTime) < CACHE_DURATION_MS) {
    // Update animation values even from cache
    return {
      ...cachedData,
      pulseIntensity: Math.sin(now / 1000) * 0.5 + 0.5,
      wavePhase: (now % 10000) / 10000,
      timestamp: now,
      lastUpdate: new Date().toISOString(),
    };
  }

  try {
    // Fetch chain health
    const healthResponse = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'chain_getHealth',
        params: [],
        id: 1,
      }),
      cache: 'no-store',
    });

    const healthData = await healthResponse.json();
    const health = healthData.result || {};

    // Calculate derived values
    const blockTime = health.block_time || 6000;
    const blockProgress = (now % blockTime) / blockTime;
    const networkLoad = Math.min(100, Math.max(0, (health.tps || 0) * 10)); // Rough estimate

    cachedData = {
      blockHeight: health.block_number || 0,
      blockTime: blockTime,
      validators: health.validators || 0,
      isConnected: health.connected !== false,
      tps: health.tps || 0,
      networkLoad: networkLoad,
      peerCount: health.peers || 0,
      finality: health.finality || 2000,
      blockProgress: blockProgress,
      pulseIntensity: Math.sin(now / 1000) * 0.5 + 0.5,
      wavePhase: (now % 10000) / 10000,
      timestamp: now,
      lastUpdate: new Date().toISOString(),
    };

    lastFetchTime = now;
    return cachedData;
  } catch (error) {
    console.error('[Spline Data API] Error fetching blockchain data:', error);
    
    // Return fallback data
    return {
      blockHeight: 0,
      blockTime: 6000,
      validators: 0,
      isConnected: false,
      tps: 0,
      networkLoad: 0,
      peerCount: 0,
      finality: 2000,
      blockProgress: (now % 6000) / 6000,
      pulseIntensity: Math.sin(now / 1000) * 0.5 + 0.5,
      wavePhase: (now % 10000) / 10000,
      timestamp: now,
      lastUpdate: new Date().toISOString(),
    };
  }
}

// GET - Main data endpoint for Spline Real-time API
export async function GET(request: NextRequest) {
  const data = await fetchBlockchainData();

  // Add CORS headers for Spline to access
  return NextResponse.json(data, {
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
      'Cache-Control': 'no-cache, no-store, must-revalidate',
    },
  });
}

// OPTIONS - CORS preflight
export async function OPTIONS() {
  return new NextResponse(null, {
    status: 204,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    },
  });
}

// POST - Webhook endpoint for Spline to send data back (optional)
export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    
    // Log received webhook data (useful for debugging Spline interactions)
    console.log('[Spline Webhook] Received:', body);
    
    // Could trigger on-chain actions here based on Spline scene interactions
    // For example: mint NFT when user completes a 3D puzzle
    
    return NextResponse.json({
      success: true,
      message: 'Webhook received',
      timestamp: Date.now(),
    }, {
      headers: {
        'Access-Control-Allow-Origin': '*',
      },
    });
  } catch (error) {
    return NextResponse.json(
      { error: 'Invalid request body' },
      { status: 400 }
    );
  }
}

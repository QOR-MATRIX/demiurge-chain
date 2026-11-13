// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * NFT Store / Query API Route
 * 
 * Queries on-chain NFT data via the blockchain RPC.
 * No longer uses file-based storage — the blockchain IS the source of truth.
 */

import { NextRequest, NextResponse } from 'next/server';

const RPC_ENDPOINT = process.env.DEMIURGE_RPC_URL || 'http://127.0.0.1:9944';

async function rpcCall(method: string, params: any[]): Promise<any> {
  const response = await fetch(RPC_ENDPOINT, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      jsonrpc: '2.0',
      id: 1,
      method,
      params,
    }),
  });
  
  const result = await response.json();
  if (result.error) {
    throw new Error(result.error.message || JSON.stringify(result.error));
  }
  return result.result;
}

// GET - Retrieve NFT(s) from blockchain
export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const tokenId = searchParams.get('tokenId');
  const owner = searchParams.get('owner');
  
  try {
    if (tokenId) {
      // Query single NFT from chain
      const tokenInfo = await rpcCall('drc369_get_token_info', [tokenId]);
      
      if (!tokenInfo) {
        return NextResponse.json({ error: 'NFT not found' }, { status: 404 });
      }
      
      return NextResponse.json(tokenInfo);
    }
    
    if (owner) {
      // Query balance for owner
      const balance = await rpcCall('drc369_balance_of', [owner]);
      return NextResponse.json({ 
        owner, 
        balance: parseInt(balance || '0'),
        note: 'Use drc369_get_token_info with specific tokenId to get full details',
      });
    }
    
    // Return total supply
    const totalSupply = await rpcCall('drc369_total_supply', []);
    return NextResponse.json({
      totalSupply: parseInt(totalSupply || '0'),
    });
    
  } catch (error) {
    console.error('[NFT Store] Query error:', error);
    return NextResponse.json(
      { error: 'Failed to query NFT data', details: error instanceof Error ? error.message : 'Unknown' },
      { status: 502 }
    );
  }
}

// POST is no longer needed — minting writes directly to the blockchain.
// Kept as a compatibility shim that returns an error.
export async function POST(request: NextRequest) {
  return NextResponse.json(
    { 
      error: 'Direct store writes are no longer supported. Use /api/nft/mint to mint NFTs on-chain.',
      migrated: true,
    },
    { status: 410 }
  );
}

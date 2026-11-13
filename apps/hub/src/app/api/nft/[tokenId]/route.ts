// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * NFT Query by Token ID
 * 
 * Queries on-chain NFT data via the blockchain RPC node.
 */

import { NextRequest, NextResponse } from 'next/server';

const RPC_ENDPOINT = process.env.DEMIURGE_RPC_URL || 'http://127.0.0.1:9944';

export async function GET(
  request: NextRequest,
  { params }: { params: Promise<{ tokenId: string }> }
) {
  const { tokenId } = await params;
  
  try {
    const response = await fetch(RPC_ENDPOINT, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        id: 1,
        method: 'drc369_get_token_info',
        params: [tokenId],
      }),
    });
    
    const result = await response.json();
    
    if (result.error) {
      return NextResponse.json(
        { error: 'Failed to query token', rpcError: result.error.message || result.error },
        { status: 502 }
      );
    }
    
    if (!result.result) {
      return NextResponse.json(
        { error: 'NFT not found', tokenId },
        { status: 404 }
      );
    }
    
    return NextResponse.json(result.result);
    
  } catch (error) {
    console.error('[NFT Query] Error:', error);
    return NextResponse.json(
      { error: 'Blockchain node unreachable', details: error instanceof Error ? error.message : 'Unknown' },
      { status: 503 }
    );
  }
}

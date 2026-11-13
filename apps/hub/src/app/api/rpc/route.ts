// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * RPC Proxy Route
 * 
 * Proxies RPC requests to the Demiurge blockchain node.
 * This avoids CORS issues when making requests from the browser.
 */

import { NextRequest, NextResponse } from 'next/server';

const RPC_ENDPOINT = process.env.DEMIURGE_RPC_URL || 'http://127.0.0.1:9944';

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    
    // Validate it's a JSON-RPC request
    if (!body.jsonrpc || !body.method) {
      return NextResponse.json(
        { error: { code: -32600, message: 'Invalid Request' } },
        { status: 400 }
      );
    }
    
    // Forward to the RPC endpoint
    const response = await fetch(RPC_ENDPOINT, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(body),
    });
    
    const data = await response.json();
    
    return NextResponse.json(data);
  } catch (error) {
    console.error('[RPC Proxy] Error:', error);
    
    return NextResponse.json(
      {
        jsonrpc: '2.0',
        id: null,
        error: {
          code: -32603,
          message: 'Internal error',
          data: error instanceof Error ? error.message : 'Unknown error',
        },
      },
      { status: 500 }
    );
  }
}

export async function OPTIONS() {
  return new NextResponse(null, {
    status: 200,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'POST, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type',
    },
  });
}

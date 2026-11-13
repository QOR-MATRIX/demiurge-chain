// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import { NextResponse } from 'next/server'

export async function GET() {
  try {
    const rpcUrl = process.env.NEXT_PUBLIC_RPC_URL || 'ws://127.0.0.1:9944'
    
    // Try to connect to blockchain RPC
    // For now, we'll use a simple HTTP check
    // In production, you'd use Polkadot.js API
    
    const startTime = Date.now()
    
    // Check if RPC endpoint is reachable
    // This is a simplified check - in production, use actual RPC calls
    const isReachable = await checkRPCReachability(rpcUrl)
    
    const latency = Date.now() - startTime

    if (isReachable) {
      // In production, fetch actual block number from RPC
      const blockNumber = await getBlockNumber(rpcUrl).catch(() => null)
      
      return NextResponse.json({
        isOnline: true,
        status: 'connected',
        blockNumber: blockNumber || 0,
        latency,
        timestamp: new Date().toISOString(),
      })
    } else {
      return NextResponse.json({
        isOnline: false,
        status: 'disconnected',
        comingSoon: false,
        timestamp: new Date().toISOString(),
      })
    }
  } catch (error: any) {
    console.error('Chain status check error:', error)
    return NextResponse.json({
      isOnline: false,
      status: 'error',
      error: error.message || 'Unknown error',
      timestamp: new Date().toISOString(),
    }, { status: 500 })
  }
}

async function checkRPCReachability(url: string): Promise<boolean> {
  try {
    // Try to make a simple RPC call
    // In production, use Polkadot.js API
    const response = await fetch('http://127.0.0.1:9944', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'chain_getHealth',
        params: [],
        id: 1,
      }),
    })
    
    return response.ok
  } catch {
    return false
  }
}

async function getBlockNumber(url: string): Promise<number | null> {
  try {
    const response = await fetch('http://127.0.0.1:9944', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        jsonrpc: '2.0',
        method: 'chain_getBlockNumber',
        params: [],
        id: 2,
      }),
    })
    
    if (!response.ok) return null
    
    const data = await response.json()
    return data.result || null
  } catch {
    return null
  }
}

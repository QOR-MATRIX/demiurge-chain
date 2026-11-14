// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import { NextRequest, NextResponse } from 'next/server'

const QOR_AUTH_URL = process.env.NEXT_PUBLIC_QOR_AUTH_URL || 'http://127.0.0.1:8080/api/v1'

export async function POST(request: NextRequest) {
  try {
    const body = await request.json()
    const { action, email, password, username } = body

    if (action === 'login') {
      // Handle login
      const response = await fetch(`${QOR_AUTH_URL}/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password }),
      })

      if (!response.ok) {
        const error = await response.json()
        return NextResponse.json(
          { error: error.message || 'Login failed' },
          { status: response.status }
        )
      }

      const data = await response.json()
      return NextResponse.json({
        success: true,
        tokens: data,
        qorId: data.qor_id,
      })
    } else if (action === 'register') {
      // Handle registration
      const response = await fetch(`${QOR_AUTH_URL}/auth/register`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, password, username }),
      })

      if (!response.ok) {
        const error = await response.json()
        return NextResponse.json(
          { error: error.message || 'Registration failed' },
          { status: response.status }
        )
      }

      const data = await response.json()
      return NextResponse.json({
        success: true,
        qorId: data.qor_id,
        message: 'Registration successful! Please check your email to verify your account.',
      })
    } else {
      return NextResponse.json(
        { error: 'Invalid action' },
        { status: 400 }
      )
    }
  } catch (error: any) {
    console.error('Sophia auth error:', error)
    return NextResponse.json(
      { error: error.message || 'Authentication failed' },
      { status: 500 }
    )
  }
}

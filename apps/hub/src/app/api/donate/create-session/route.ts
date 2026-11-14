// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import { NextRequest, NextResponse } from 'next/server';
import { qorAuth } from '@demiurge/qor-sdk';
import { createDonationCheckoutSession } from '@/lib/stripe';
import { getTierFromAmount, getTierByLevel, formatAmount } from '@/lib/donation-tiers';

const QOR_AUTH_URL = process.env.NEXT_PUBLIC_QOR_AUTH_URL || 'http://localhost:8080';

/**
 * POST /api/donate/create-session
 * Creates a Stripe Checkout Session for one-time donation
 */
export async function POST(request: NextRequest) {
  try {
    // Verify authentication
    const authHeader = request.headers.get('Authorization');
    if (!authHeader) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { amountCents } = body;

    // Validate amount
    if (!amountCents || typeof amountCents !== 'number' || amountCents < 100) {
      return NextResponse.json(
        { error: 'Minimum donation is $1.00' },
        { status: 400 }
      );
    }

    if (amountCents > 10000000) { // $100,000 max
      return NextResponse.json(
        { error: 'Maximum single donation is $100,000' },
        { status: 400 }
      );
    }

    // Get user info from QOR Auth
    const userResponse = await fetch(`${QOR_AUTH_URL}/api/v1/auth/me`, {
      headers: { 'Authorization': authHeader },
    });

    if (!userResponse.ok) {
      return NextResponse.json({ error: 'Failed to verify user' }, { status: 401 });
    }

    const userData = await userResponse.json();
    const userId = userData.id;
    const userEmail = userData.email;

    // Get user's current donation status to calculate new tier
    const statusResponse = await fetch(`${QOR_AUTH_URL}/api/v1/donations/status`, {
      headers: { 'Authorization': authHeader },
    });

    let currentLifetimeCents = 0;
    if (statusResponse.ok) {
      const statusData = await statusResponse.json();
      currentLifetimeCents = statusData.lifetime_cents || 0;
    }

    // Calculate new tier based on lifetime + this donation
    const newLifetimeCents = currentLifetimeCents + amountCents;
    const newTierLevel = getTierFromAmount(newLifetimeCents);
    const tier = getTierByLevel(newTierLevel);

    if (!tier) {
      return NextResponse.json(
        { error: 'Invalid donation amount' },
        { status: 400 }
      );
    }

    // Create Stripe Checkout Session
    const origin = request.headers.get('origin') || 'https://qorsync.dev';
    const session = await createDonationCheckoutSession({
      userId,
      userEmail,
      amountCents,
      tierLevel: newTierLevel,
      successUrl: `${origin}/donate/success`,
      cancelUrl: `${origin}/donate?cancelled=true`,
    });

    if (!session) {
      return NextResponse.json(
        { error: 'Failed to create checkout session. Payment processing may be unavailable.' },
        { status: 503 }
      );
    }

    // Return session info
    return NextResponse.json({
      sessionId: session.id,
      url: session.url,
      tier: {
        level: newTierLevel,
        name: tier.name,
        cgtReward: tier.cgtReward,
        stakingBonus: tier.stakingBonusBps,
        xpBonus: tier.xpRateBonusBps,
        freeMints: tier.freeMints,
      },
      amount: formatAmount(amountCents),
      newLifetimeTotal: formatAmount(newLifetimeCents),
    });
  } catch (error: any) {
    console.error('Create donation session error:', error);
    return NextResponse.json(
      { error: error.message || 'Internal server error' },
      { status: 500 }
    );
  }
}

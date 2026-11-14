// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import { NextRequest, NextResponse } from 'next/server';
import { createSubscriptionCheckoutSession } from '@/lib/stripe';
import { SUBSCRIPTION_TIERS, getSubscriptionTierByLevel, getTierByLevel, formatAmount } from '@/lib/donation-tiers';

const QOR_AUTH_URL = process.env.NEXT_PUBLIC_QOR_AUTH_URL || 'http://localhost:8080';

// Stripe Price IDs for subscription tiers (configured in Stripe Dashboard)
// These need to be set up as bi-weekly recurring prices
const SUBSCRIPTION_PRICE_IDS: Record<number, string> = {
  1: process.env.STRIPE_PRICE_SUB_TIER_1 || 'price_supporter_biweekly',
  2: process.env.STRIPE_PRICE_SUB_TIER_2 || 'price_champion_biweekly',
  3: process.env.STRIPE_PRICE_SUB_TIER_3 || 'price_guardian_biweekly',
  4: process.env.STRIPE_PRICE_SUB_TIER_4 || 'price_archon_biweekly',
  5: process.env.STRIPE_PRICE_SUB_TIER_5 || 'price_godsent_biweekly',
};

/**
 * POST /api/donate/create-subscription
 * Creates a Stripe Checkout Session for bi-weekly subscription
 */
export async function POST(request: NextRequest) {
  try {
    // Verify authentication
    const authHeader = request.headers.get('Authorization');
    if (!authHeader) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    const body = await request.json();
    const { tierLevel } = body;

    // Validate tier
    if (!tierLevel || tierLevel < 1 || tierLevel > 5) {
      return NextResponse.json(
        { error: 'Invalid subscription tier (1-5)' },
        { status: 400 }
      );
    }

    const subscriptionTier = getSubscriptionTierByLevel(tierLevel);
    if (!subscriptionTier) {
      return NextResponse.json(
        { error: 'Subscription tier not found' },
        { status: 400 }
      );
    }

    const priceId = SUBSCRIPTION_PRICE_IDS[tierLevel];
    if (!priceId || priceId.startsWith('price_')) {
      // Price ID not configured - return helpful error
      return NextResponse.json(
        { 
          error: 'Subscription pricing not configured. Please contact support.',
          details: 'Stripe Price IDs need to be configured for subscriptions.'
        },
        { status: 503 }
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

    // Check if user already has an active subscription
    const statusResponse = await fetch(`${QOR_AUTH_URL}/api/v1/donations/status`, {
      headers: { 'Authorization': authHeader },
    });

    if (statusResponse.ok) {
      const statusData = await statusResponse.json();
      if (statusData.subscription_status === 'active') {
        return NextResponse.json(
          { 
            error: 'You already have an active subscription. Please manage your existing subscription first.',
            currentTier: statusData.subscription_tier,
          },
          { status: 409 }
        );
      }
    }

    // Get the effective tier perks they'll receive
    const effectiveTier = getTierByLevel(subscriptionTier.effectiveTier);

    // Create Stripe Subscription Checkout Session
    const origin = request.headers.get('origin') || 'https://qorsync.dev';
    const session = await createSubscriptionCheckoutSession({
      userId,
      userEmail,
      subscriptionTierLevel: tierLevel,
      priceId,
      successUrl: `${origin}/donate/success?type=subscription`,
      cancelUrl: `${origin}/donate?cancelled=true`,
    });

    if (!session) {
      return NextResponse.json(
        { error: 'Failed to create subscription session. Payment processing may be unavailable.' },
        { status: 503 }
      );
    }

    // Return session info
    return NextResponse.json({
      sessionId: session.id,
      url: session.url,
      subscription: {
        level: tierLevel,
        name: subscriptionTier.name,
        biWeeklyAmount: formatAmount(subscriptionTier.biWeeklyAmount),
        cgtPerCycle: subscriptionTier.cgtPerCycle,
        freeMintsCycle: subscriptionTier.freeMintsCycle,
      },
      effectivePerks: effectiveTier ? {
        tierName: effectiveTier.name,
        stakingBonus: effectiveTier.stakingBonusBps,
        xpBonus: effectiveTier.xpRateBonusBps,
        energyDiscount: effectiveTier.energyDiscountBps,
        chatPrivileges: effectiveTier.chatPrivileges,
      } : null,
    });
  } catch (error: any) {
    console.error('Create subscription session error:', error);
    return NextResponse.json(
      { error: error.message || 'Internal server error' },
      { status: 500 }
    );
  }
}

/**
 * GET /api/donate/create-subscription
 * Returns available subscription tiers
 */
export async function GET() {
  const tiers = SUBSCRIPTION_TIERS.map(tier => {
    const effectiveTier = getTierByLevel(tier.effectiveTier);
    return {
      level: tier.level,
      name: tier.name,
      biWeeklyAmount: tier.biWeeklyAmount,
      biWeeklyAmountFormatted: formatAmount(tier.biWeeklyAmount),
      monthlyEquivalent: formatAmount(tier.biWeeklyAmount * 2.17), // ~2.17 bi-weekly periods per month
      cgtPerCycle: tier.cgtPerCycle,
      freeMintsCycle: tier.freeMintsCycle,
      effectiveTierName: effectiveTier?.name || 'Unknown',
      effectivePerks: effectiveTier ? {
        stakingBonus: `+${effectiveTier.stakingBonusBps / 100}%`,
        xpBonus: `+${effectiveTier.xpRateBonusBps / 100}%`,
        energyDiscount: effectiveTier.energyDiscountBps > 0 ? `${effectiveTier.energyDiscountBps / 100}%` : 'None',
        freeMints: effectiveTier.freeMints,
        chatPrivileges: effectiveTier.chatPrivileges,
      } : null,
    };
  });

  return NextResponse.json({ tiers });
}

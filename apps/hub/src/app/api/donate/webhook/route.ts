// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
import { NextRequest, NextResponse } from 'next/server';
import { verifyWebhookSignature, stripe } from '@/lib/stripe';
import { 
  getTierByLevel, 
  getSubscriptionTierByLevel,
  calculateCgtReward,
  calculateFreeMints 
} from '@/lib/donation-tiers';
import { demiurgeRpc } from '@/lib/demiurge-rpc';
import { treasury } from '@/lib/treasury';
import Stripe from 'stripe';

const QOR_AUTH_URL = process.env.NEXT_PUBLIC_QOR_AUTH_URL || 'http://localhost:8080';

/**
 * POST /api/donate/webhook
 * Handles Stripe webhook events for donations and subscriptions
 */
export async function POST(request: NextRequest) {
  try {
    const body = await request.text();
    const signature = request.headers.get('stripe-signature');

    if (!signature) {
      return NextResponse.json({ error: 'Missing signature' }, { status: 400 });
    }

    // Verify webhook signature
    const event = verifyWebhookSignature(body, signature);
    if (!event) {
      return NextResponse.json({ error: 'Invalid signature' }, { status: 400 });
    }

    // Handle different event types
    switch (event.type) {
      case 'checkout.session.completed':
        await handleCheckoutComplete(event.data.object as Stripe.Checkout.Session);
        break;

      case 'invoice.payment_succeeded':
        await handleInvoicePaymentSucceeded(event.data.object as Stripe.Invoice);
        break;

      case 'invoice.payment_failed':
        await handleInvoicePaymentFailed(event.data.object as Stripe.Invoice);
        break;

      case 'customer.subscription.created':
        await handleSubscriptionCreated(event.data.object as Stripe.Subscription);
        break;

      case 'customer.subscription.updated':
        await handleSubscriptionUpdated(event.data.object as Stripe.Subscription);
        break;

      case 'customer.subscription.deleted':
        await handleSubscriptionDeleted(event.data.object as Stripe.Subscription);
        break;

      case 'charge.refunded':
        await handleChargeRefunded(event.data.object as Stripe.Charge);
        break;

      default:
        // Unhandled event type - consider logging to monitoring service
        break;
    }

    return NextResponse.json({ received: true });
  } catch (error: any) {
    console.error('[Webhook] Error:', error);
    return NextResponse.json(
      { error: error.message || 'Webhook handler failed' },
      { status: 500 }
    );
  }
}

/**
 * Handle completed checkout session (one-time donation or subscription start)
 */
async function handleCheckoutComplete(session: Stripe.Checkout.Session) {
  const userId = session.metadata?.user_id;
  const donationType = session.metadata?.donation_type;

  if (!userId) {
    console.error('[Webhook] No user_id in session metadata');
    return;
  }

  if (donationType === 'one_time') {
    await processOneTimeDonation(session);
  } else if (donationType === 'subscription') {
    // Subscription is handled by subscription.created event
  }
}

/**
 * Process one-time donation
 */
async function processOneTimeDonation(session: Stripe.Checkout.Session) {
  const userId = session.metadata?.user_id;
  const tierLevel = parseInt(session.metadata?.tier_level || '0');
  const amountCents = session.amount_total || 0;

  if (!userId || !tierLevel || !amountCents) {
    console.error('[Webhook] Missing data for one-time donation');
    return;
  }

  const tier = getTierByLevel(tierLevel);
  if (!tier) return;

  try {
    // 1. Update donor profile in QOR Auth
    const updateResponse = await fetch(`${QOR_AUTH_URL}/api/v1/donations/process`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Webhook-Secret': process.env.WEBHOOK_INTERNAL_SECRET || '',
      },
      body: JSON.stringify({
        user_id: userId,
        amount_cents: amountCents,
        stripe_session_id: session.id,
        stripe_payment_intent: session.payment_intent,
        tier_level: tierLevel,
        donation_type: 'one_time',
      }),
    });

    if (!updateResponse.ok) {
      console.error('[Webhook] Failed to update donor profile');
    }

    const donorData = await updateResponse.json();

    // 2. Transfer CGT reward
    const cgtReward = calculateCgtReward(tierLevel, donorData.previous_cgt_received || 0);
    if (cgtReward > 0) {
      try {
        const userAddress = donorData.on_chain_address;
        if (userAddress) {
          // Transfer CGT from treasury
          const txHash = await treasury.transferCGT(userAddress, cgtReward, `Donation tier ${tierLevel} reward`);
          if (txHash) {
            // CGT transfer successful
          } else {
            console.warn(`[Webhook] CGT transfer pending - treasury not available`);
          }
        }
      } catch (transferError) {
        console.error('[Webhook] CGT transfer failed:', transferError);
      }
    }

    // 3. Mint/upgrade donor badge NFT
    if (!donorData.badge_nft_uuid || donorData.tier_upgraded) {
      await mintOrUpgradeDonorBadge(userId, tierLevel, donorData);
    }
  } catch (error) {
    console.error('[Webhook] Error processing one-time donation:', error);
  }
}

/**
 * Handle subscription invoice payment succeeded (recurring)
 */
async function handleInvoicePaymentSucceeded(invoice: Stripe.Invoice) {
  // Get subscription ID from invoice - cast to any for Stripe API compatibility
  const invoiceData = invoice as any;
  const subscriptionId = typeof invoiceData.subscription === 'string' 
    ? invoiceData.subscription 
    : invoiceData.subscription?.id;
  
  if (!subscriptionId) return;

  const subscription = await stripe?.subscriptions.retrieve(subscriptionId);
  if (!subscription) return;

  const userId = subscription.metadata?.user_id;
  const subscriptionTierLevel = parseInt(subscription.metadata?.subscription_tier || '0');

  if (!userId || !subscriptionTierLevel) return;

  const subTier = getSubscriptionTierByLevel(subscriptionTierLevel);
  if (!subTier) return;

  try {
    // 1. Record subscription payment
    await fetch(`${QOR_AUTH_URL}/api/v1/donations/subscription-payment`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Webhook-Secret': process.env.WEBHOOK_INTERNAL_SECRET || '',
      },
      body: JSON.stringify({
        user_id: userId,
        stripe_invoice_id: invoice.id,
        subscription_tier: subscriptionTierLevel,
        amount_cents: invoice.amount_paid,
        cgt_distributed: subTier.cgtPerCycle,
        free_mints_granted: subTier.freeMintsCycle,
      }),
    });

    // 2. Transfer CGT for this cycle
    const userResponse = await fetch(`${QOR_AUTH_URL}/api/v1/users/${userId}`, {
      headers: { 'X-Webhook-Secret': process.env.WEBHOOK_INTERNAL_SECRET || '' },
    });

    if (userResponse.ok) {
      const userData = await userResponse.json();
      if (userData.on_chain_address && subTier.cgtPerCycle > 0) {
        try {
          const txHash = await treasury.transferCGT(
            userData.on_chain_address, 
            subTier.cgtPerCycle,
            `Subscription tier ${subscriptionTierLevel} bi-weekly reward`
          );
          if (txHash) {
            // Subscription CGT transfer successful
          } else {
            console.warn(`[Webhook] Subscription CGT transfer pending - treasury not available`);
          }
        } catch (transferError) {
          console.error('[Webhook] Subscription CGT transfer failed:', transferError);
        }
      }
    }
  } catch (error) {
    console.error('[Webhook] Error processing subscription payment:', error);
  }
}

/**
 * Handle subscription invoice payment failed
 */
async function handleInvoicePaymentFailed(invoice: Stripe.Invoice) {
  // Get subscription ID from invoice - cast to any for Stripe API compatibility
  const invoiceData = invoice as any;
  const subscriptionId = typeof invoiceData.subscription === 'string' 
    ? invoiceData.subscription 
    : invoiceData.subscription?.id;
  
  if (!subscriptionId) return;

  const subscription = await stripe?.subscriptions.retrieve(subscriptionId);
  if (!subscription) return;

  const userId = subscription.metadata?.user_id;
  if (!userId) return;

  try {
    await fetch(`${QOR_AUTH_URL}/api/v1/donations/subscription-failed`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Webhook-Secret': process.env.WEBHOOK_INTERNAL_SECRET || '',
      },
      body: JSON.stringify({
        user_id: userId,
        stripe_invoice_id: invoice.id,
        attempt_count: invoice.attempt_count,
      }),
    });
  } catch (error) {
    console.error('[Webhook] Error handling payment failure:', error);
  }
}

/**
 * Handle subscription created
 */
async function handleSubscriptionCreated(subscription: Stripe.Subscription) {
  // Cast to any for Stripe API compatibility
  const sub = subscription as any;
  const userId = sub.metadata?.user_id;
  const subscriptionTierLevel = parseInt(sub.metadata?.subscription_tier || '0');

  if (!userId || !subscriptionTierLevel) return;

  try {
    await fetch(`${QOR_AUTH_URL}/api/v1/donations/subscription-created`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Webhook-Secret': process.env.WEBHOOK_INTERNAL_SECRET || '',
      },
      body: JSON.stringify({
        user_id: userId,
        stripe_subscription_id: sub.id,
        subscription_tier: subscriptionTierLevel,
        current_period_end: new Date(sub.current_period_end * 1000).toISOString(),
      }),
    });
  } catch (error) {
    console.error('[Webhook] Error handling subscription created:', error);
  }
}

/**
 * Handle subscription updated (tier change, pause, etc.)
 */
async function handleSubscriptionUpdated(subscription: Stripe.Subscription) {
  // Cast to any for Stripe API compatibility
  const sub = subscription as any;
  const userId = sub.metadata?.user_id;
  if (!userId) return;

  try {
    await fetch(`${QOR_AUTH_URL}/api/v1/donations/subscription-updated`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Webhook-Secret': process.env.WEBHOOK_INTERNAL_SECRET || '',
      },
      body: JSON.stringify({
        user_id: userId,
        stripe_subscription_id: sub.id,
        status: sub.status,
        cancel_at_period_end: sub.cancel_at_period_end,
        paused: sub.pause_collection !== null,
        current_period_end: new Date(sub.current_period_end * 1000).toISOString(),
      }),
    });
  } catch (error) {
    console.error('[Webhook] Error handling subscription update:', error);
  }
}

/**
 * Handle subscription deleted/cancelled
 */
async function handleSubscriptionDeleted(subscription: Stripe.Subscription) {
  // Cast to any for Stripe API compatibility
  const sub = subscription as any;
  const userId = sub.metadata?.user_id;
  if (!userId) return;

  try {
    await fetch(`${QOR_AUTH_URL}/api/v1/donations/subscription-cancelled`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Webhook-Secret': process.env.WEBHOOK_INTERNAL_SECRET || '',
      },
      body: JSON.stringify({
        user_id: userId,
        stripe_subscription_id: sub.id,
        grace_period_end: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toISOString(), // 7 days
      }),
    });
  } catch (error) {
    console.error('[Webhook] Error handling subscription cancellation:', error);
  }
}

/**
 * Handle charge refunded
 */
async function handleChargeRefunded(charge: Stripe.Charge) {
  const paymentIntent = charge.payment_intent as string;
  if (!paymentIntent) return;

  try {
    await fetch(`${QOR_AUTH_URL}/api/v1/donations/refunded`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'X-Webhook-Secret': process.env.WEBHOOK_INTERNAL_SECRET || '',
      },
      body: JSON.stringify({
        stripe_payment_intent: paymentIntent,
        refund_amount_cents: charge.amount_refunded,
      }),
    });
  } catch (error) {
    console.error('[Webhook] Error handling refund:', error);
  }
}

/**
 * Mint or upgrade donor badge NFT
 */
async function mintOrUpgradeDonorBadge(
  userId: string, 
  tierLevel: number, 
  donorData: any
) {
  const tier = getTierByLevel(tierLevel);
  if (!tier) return;

  const nftMetadata = {
    name: `Demiurge ${tier.name} Badge`,
    description: `Official ${tier.name} supporter badge for the Demiurge ecosystem`,
    image: tier.badgeImage,
    attributes: [
      { trait_type: 'Tier', value: tier.name },
      { trait_type: 'Tier Level', value: tierLevel.toString() },
      { trait_type: 'Soulbound', value: 'true' },
      { trait_type: 'Lifetime Donated', value: `$${(donorData.lifetime_cents / 100).toFixed(2)}` },
    ],
    dynamic_state: {
      free_mints_remaining: donorData.free_mints_remaining || tier.freeMints,
      staking_bonus_bps: tier.stakingBonusBps,
      xp_rate_bonus_bps: tier.xpRateBonusBps,
      gas_discount_bps: tier.energyDiscountBps,
      chat_privileges: tier.chatPrivileges,
      cgt_received: donorData.total_cgt_received || tier.cgtReward,
      upgraded_at: new Date().toISOString(),
    },
  };

  try {
    if (donorData.badge_nft_uuid) {
      // Update existing badge via fetch to RPC endpoint
      const rpcUrl = process.env.NEXT_PUBLIC_BLOCKCHAIN_RPC_URL || 'http://127.0.0.1:9944';
      await fetch(rpcUrl, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          jsonrpc: '2.0',
          method: 'drc369_updateState',
          params: [donorData.badge_nft_uuid, nftMetadata.dynamic_state],
          id: Date.now(),
        }),
      });
    } else {
      // Mint new badge
      const userAddress = donorData.on_chain_address;
      if (userAddress) {
        const metadataUri = `data:application/json;base64,${Buffer.from(JSON.stringify(nftMetadata)).toString('base64')}`;
        const result = await demiurgeRpc.mintNFT(userAddress, metadataUri, 0, '');
        
        if (result.success && result.tokenId) {
          // Update donor profile with badge UUID
          await fetch(`${QOR_AUTH_URL}/api/v1/donations/set-badge`, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'X-Webhook-Secret': process.env.WEBHOOK_INTERNAL_SECRET || '',
            },
            body: JSON.stringify({
              user_id: userId,
              badge_nft_uuid: result.tokenId,
            }),
          });
        }
      }
    }
  } catch (error) {
    console.error('[Webhook] Error minting/updating badge:', error);
  }
}

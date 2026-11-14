// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Stripe Integration for Demiurge Donations
 * 
 * Server-side Stripe utilities for payment processing.
 */

import Stripe from 'stripe';

// Initialize Stripe with secret key
const stripeSecretKey = process.env.STRIPE_SECRET_KEY;

if (!stripeSecretKey && process.env.NODE_ENV === 'production') {
  console.warn('STRIPE_SECRET_KEY not configured - payment processing disabled');
}

export const stripe = stripeSecretKey 
  ? new Stripe(stripeSecretKey, {
      apiVersion: '2025-12-15.clover',
      typescript: true,
    })
  : null;

/**
 * Verify Stripe webhook signature
 */
export function verifyWebhookSignature(
  payload: string | Buffer,
  signature: string
): Stripe.Event | null {
  if (!stripe) return null;
  
  const webhookSecret = process.env.STRIPE_WEBHOOK_SECRET;
  if (!webhookSecret) {
    console.error('STRIPE_WEBHOOK_SECRET not configured');
    return null;
  }
  
  try {
    return stripe.webhooks.constructEvent(payload, signature, webhookSecret);
  } catch (err) {
    console.error('Webhook signature verification failed:', err);
    return null;
  }
}

/**
 * Create a Stripe Checkout Session for one-time donation
 */
export async function createDonationCheckoutSession(params: {
  userId: string;
  userEmail: string;
  amountCents: number;
  tierLevel: number;
  successUrl: string;
  cancelUrl: string;
}): Promise<Stripe.Checkout.Session | null> {
  if (!stripe) return null;
  
  const { userId, userEmail, amountCents, tierLevel, successUrl, cancelUrl } = params;
  
  try {
    const session = await stripe.checkout.sessions.create({
      mode: 'payment',
      customer_email: userEmail,
      line_items: [
        {
          price_data: {
            currency: 'usd',
            product_data: {
              name: `Demiurge Donation - Tier ${tierLevel}`,
              description: `Support the Demiurge ecosystem and receive exclusive perks`,
              images: ['https://qorsync.dev/donation-hero.png'],
            },
            unit_amount: amountCents,
          },
          quantity: 1,
        },
      ],
      metadata: {
        user_id: userId,
        tier_level: tierLevel.toString(),
        donation_type: 'one_time',
      },
      success_url: `${successUrl}?session_id={CHECKOUT_SESSION_ID}`,
      cancel_url: cancelUrl,
      payment_intent_data: {
        metadata: {
          user_id: userId,
          tier_level: tierLevel.toString(),
        },
      },
    });
    
    return session;
  } catch (err) {
    console.error('Failed to create checkout session:', err);
    return null;
  }
}

/**
 * Create a Stripe Subscription Checkout Session
 */
export async function createSubscriptionCheckoutSession(params: {
  userId: string;
  userEmail: string;
  subscriptionTierLevel: number;
  priceId: string;
  successUrl: string;
  cancelUrl: string;
}): Promise<Stripe.Checkout.Session | null> {
  if (!stripe) return null;
  
  const { userId, userEmail, subscriptionTierLevel, priceId, successUrl, cancelUrl } = params;
  
  try {
    const session = await stripe.checkout.sessions.create({
      mode: 'subscription',
      customer_email: userEmail,
      line_items: [
        {
          price: priceId,
          quantity: 1,
        },
      ],
      metadata: {
        user_id: userId,
        subscription_tier: subscriptionTierLevel.toString(),
        donation_type: 'subscription',
      },
      subscription_data: {
        metadata: {
          user_id: userId,
          subscription_tier: subscriptionTierLevel.toString(),
        },
      },
      success_url: `${successUrl}?session_id={CHECKOUT_SESSION_ID}`,
      cancel_url: cancelUrl,
    });
    
    return session;
  } catch (err) {
    console.error('Failed to create subscription session:', err);
    return null;
  }
}

/**
 * Cancel a subscription
 */
export async function cancelSubscription(
  subscriptionId: string,
  cancelAtPeriodEnd: boolean = true
): Promise<Stripe.Subscription | null> {
  if (!stripe) return null;
  
  try {
    if (cancelAtPeriodEnd) {
      // Cancel at end of billing period (allows grace period)
      return await stripe.subscriptions.update(subscriptionId, {
        cancel_at_period_end: true,
      });
    } else {
      // Immediate cancellation
      return await stripe.subscriptions.cancel(subscriptionId);
    }
  } catch (err) {
    console.error('Failed to cancel subscription:', err);
    return null;
  }
}

/**
 * Pause a subscription (by updating to pause collection)
 */
export async function pauseSubscription(
  subscriptionId: string
): Promise<Stripe.Subscription | null> {
  if (!stripe) return null;
  
  try {
    return await stripe.subscriptions.update(subscriptionId, {
      pause_collection: {
        behavior: 'void',
      },
    });
  } catch (err) {
    console.error('Failed to pause subscription:', err);
    return null;
  }
}

/**
 * Resume a paused subscription
 */
export async function resumeSubscription(
  subscriptionId: string
): Promise<Stripe.Subscription | null> {
  if (!stripe) return null;
  
  try {
    return await stripe.subscriptions.update(subscriptionId, {
      pause_collection: null,
    });
  } catch (err) {
    console.error('Failed to resume subscription:', err);
    return null;
  }
}

/**
 * Get Stripe Customer Portal URL for subscription management
 */
export async function createCustomerPortalSession(
  customerId: string,
  returnUrl: string
): Promise<string | null> {
  if (!stripe) return null;
  
  try {
    const session = await stripe.billingPortal.sessions.create({
      customer: customerId,
      return_url: returnUrl,
    });
    return session.url;
  } catch (err) {
    console.error('Failed to create portal session:', err);
    return null;
  }
}

/**
 * Retrieve subscription details
 */
export async function getSubscription(
  subscriptionId: string
): Promise<Stripe.Subscription | null> {
  if (!stripe) return null;
  
  try {
    return await stripe.subscriptions.retrieve(subscriptionId);
  } catch (err) {
    console.error('Failed to retrieve subscription:', err);
    return null;
  }
}

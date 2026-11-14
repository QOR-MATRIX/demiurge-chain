// Frozen legacy code: domain cleanup only; loopback examples are not live integrations (ADR-130).
/**
 * Real-time blockchain data hook
 * Replaces all mock data with actual RPC queries
 */

import { useState, useEffect } from 'react';
import { DemiurgeClient } from '@demiurge/sdk';

const RPC_URL = process.env.NEXT_PUBLIC_RPC_URL || 'http://127.0.0.1:9944';

const client = new DemiurgeClient({ endpoint: RPC_URL });

export function useBlockNumber() {
  const [blockNumber, setBlockNumber] = useState<number>(0);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let mounted = true;
    let interval: NodeJS.Timeout;

    const fetchBlockNumber = async () => {
      try {
        const block = await client.getBlockNumber();
        if (mounted) {
          setBlockNumber(block);
          setError(null);
        }
      } catch (err) {
        if (mounted) {
          setError(err as Error);
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    fetchBlockNumber();
    interval = setInterval(fetchBlockNumber, 2000); // Poll every 2s

    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, []);

  return { blockNumber, loading, error };
}

export function useBalance(address?: string) {
  const [balance, setBalance] = useState<string>('0');
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!address) {
      setLoading(false);
      return;
    }

    let mounted = true;
    let interval: NodeJS.Timeout;

    const fetchBalance = async () => {
      try {
        const bal = await client.getBalance(address);
        if (mounted) {
          setBalance(bal);
          setError(null);
        }
      } catch (err) {
        if (mounted) {
          setError(err as Error);
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    fetchBalance();
    interval = setInterval(fetchBalance, 5000); // Poll every 5s

    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [address]);

  return { balance, loading, error };
}

export function useEnergy(address?: string) {
  const [energy, setEnergy] = useState(100);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    if (!address) {
      setLoading(false);
      return;
    }

    let mounted = true;
    let interval: NodeJS.Timeout;

    const fetchEnergy = async () => {
      try {
        const energyData = await client.getEnergy(address);
        if (mounted) {
          setEnergy(energyData.current);
          setError(null);
        }
      } catch (err) {
        if (mounted) {
          setError(err as Error);
        }
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    };

    fetchEnergy();
    interval = setInterval(fetchEnergy, 5000); // Poll every 5s

    return () => {
      mounted = false;
      clearInterval(interval);
    };
  }, [address]);

  return { energy, loading, error };
}

export function useValidators() {
  const [validators, setValidators] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let mounted = true;

    const fetchValidators = async () => {
      try {
        // TODO: Implement getValidators in SDK
        // const vals = await client.getValidators();
        // For now, return empty until RPC method is implemented
        if (mounted) {
          setValidators([]);
          setError(null);
          setLoading(false);
        }
      } catch (err) {
        if (mounted) {
          setError(err as Error);
          setLoading(false);
        }
      }
    };

    fetchValidators();

    return () => {
      mounted = false;
    };
  }, []);

  return { validators, loading, error };
}

export function useAgents(ownerAddress?: string) {
  const [agents, setAgents] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let mounted = true;

    const fetchAgents = async () => {
      try {
        // TODO: Implement getAgents RPC method
        // const agentData = await client.getAgents(ownerAddress);
        // For now, return empty array
        if (mounted) {
          setAgents([]);
          setError(null);
          setLoading(false);
        }
      } catch (err) {
        if (mounted) {
          setError(err as Error);
          setLoading(false);
        }
      }
    };

    fetchAgents();

    return () => {
      mounted = false;
    };
  }, [ownerAddress]);

  return { agents, loading, error };
}

export function useBounties() {
  const [bounties, setBounties] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let mounted = true;

    const fetchBounties = async () => {
      try {
        // TODO: Implement getBounties RPC method
        // const bountyData = await client.getBounties();
        // For now, return empty array
        if (mounted) {
          setBounties([]);
          setError(null);
          setLoading(false);
        }
      } catch (err) {
        if (mounted) {
          setError(err as Error);
          setLoading(false);
        }
      }
    };

    fetchBounties();

    return () => {
      mounted = false;
    };
  }, []);

  return { bounties, loading, error };
}

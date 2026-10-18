/**
 * The CGT Vault.
 *
 * Holdings, accounts, send and receive. Two deliberate choices:
 *
 * - **The amount is parsed by the host, not here.** Typing in the send field
 *   round-trips to Rust, which returns the exact ember value it would transfer.
 *   The preview the user confirms is therefore the number that gets signed, not
 *   a JavaScript approximation of it. `u128` does not fit in a `number`, and
 *   floating point has no business anywhere near a balance.
 * - **Sending is a two-stage act.** You compose, then you confirm against a
 *   summary showing the exact recipient and amount. A transfer cannot be undone,
 *   so a single click should not be able to cause one.
 */

import { useEffect, useState } from 'react';
import {
  ArrowDownLeft,
  ArrowUpRight,
  Check,
  Copy,
  Loader2,
  RefreshCw,
} from 'lucide-react';

import { chain, explain, looksLikeAddress, shortAddress, type Balance } from '../lib/ipc';
import { selectActiveAccount, selectActiveBalance, useQor } from '../state/store';
import { ClaimGrant } from './Nexus';
import { Field, Panel, ViewHeader } from './parts';

type Mode = 'idle' | 'send' | 'receive';

export function VaultView() {
  const accounts = useQor((s) => s.accounts);
  const balances = useQor((s) => s.balances);
  const account = useQor(selectActiveAccount);
  const balance = useQor(selectActiveBalance);
  const token = useQor((s) => s.token);
  const setActiveAddress = useQor((s) => s.setActiveAddress);
  const refreshAllBalances = useQor((s) => s.refreshAllBalances);

  const [mode, setMode] = useState<Mode>('idle');
  const [refreshing, setRefreshing] = useState(false);

  const refresh = async () => {
    setRefreshing(true);
    await refreshAllBalances();
    setRefreshing(false);
  };

  return (
    <div className="flex h-full flex-col overflow-hidden">
      <ViewHeader
        eyebrow={token?.name ?? 'Creator God Token'}
        title="Vault"
        body="Your keys are held by the launcher process and never reach this interface. It can ask them to sign; it cannot read them."
        action={
          <>
            <button type="button" className="btn" onClick={() => void refresh()}>
              <RefreshCw size={13} className={refreshing ? 'animate-spin' : ''} />
              Refresh
            </button>
          </>
        }
      />

      <div className="flex min-h-0 flex-1">
        {/* Accounts */}
        <aside className="flex w-72 flex-none flex-col border-r border-edge">
          <div className="flex items-center justify-between px-5 py-4">
            {/* One account per QOR ID (ADR-099): another QOR ID is another profile, with its own wallet. */}
            <span className="eyebrow">Account</span>
          </div>

          <ul className="flex-1 overflow-y-auto px-3 pb-3">
            {accounts.map((a) => {
              const active = a.address === account?.address;
              const accountBalance = balances[a.address];
              return (
                <li key={a.address}>
                  <button
                    type="button"
                    onClick={() => setActiveAddress(a.address)}
                    className={`mb-1 flex w-full flex-col items-start gap-1 border-l-2 px-3 py-2.5 text-left transition-colors duration-150 ${
                      active
                        ? 'border-accent bg-surface text-ink'
                        : 'border-transparent text-ink-muted hover:bg-raised'
                    }`}
                  >
                    <span className="text-ui font-semibold">{a.label}</span>
                    <span className="numeric text-micro text-ink-faint">
                      {shortAddress(a.address, 10, 6)}
                    </span>
                    <span className="numeric text-caption text-accent">
                      {accountBalance?.display ?? '…'}
                    </span>
                  </button>
                </li>
              );
            })}
          </ul>
        </aside>

        {/* Detail */}
        <section className="flex-1 overflow-y-auto p-8">
          {!account ? (
            <p className="text-ink-muted">No account selected.</p>
          ) : (
            <>
              <ClaimGrant />
              <Panel className="mb-6 p-6">
                <p className="eyebrow mb-3">{account.label}</p>
                <p className="mb-5 flex items-baseline gap-2">
                  <span className="numeric text-figure leading-none text-ink">
                    {balance?.cgt ?? '—'}
                  </span>
                  <span className="heading text-body text-accent">
                    {token?.symbol ?? 'CGT'}
                  </span>
                </p>
                {balance?.in_sparks && (
                  <p className="numeric -mt-3 mb-5 text-caption text-ink-muted" data-vault-sparks>
                    {balance.in_sparks}
                  </p>
                )}

                <div className="flex gap-2">
                  <button
                    type="button"
                    className="btn btn-primary"
                    onClick={() => setMode(mode === 'send' ? 'idle' : 'send')}
                  >
                    <ArrowUpRight size={13} />
                    Send
                  </button>
                  <button
                    type="button"
                    className="btn"
                    onClick={() => setMode(mode === 'receive' ? 'idle' : 'receive')}
                  >
                    <ArrowDownLeft size={13} />
                    Receive
                  </button>
                </div>
              </Panel>

              {mode === 'send' && (
                <SendPanel from={account.address} onDone={() => setMode('idle')} />
              )}
              {mode === 'receive' && <ReceivePanel address={account.address} />}

              <Panel className="mt-6 p-5">
                <p className="eyebrow mb-3">Derivation</p>
                <dl className="grid grid-cols-[110px_1fr] gap-y-2 text-ui">
                  <dt className="text-ink-muted">Path</dt>
                  <dd className="numeric selectable text-ink">
                    {account.path === '' ? 'the phrase itself' : account.path}
                  </dd>
                  <dt className="text-ink-muted">Address</dt>
                  <dd className="numeric selectable break-all text-ink">{account.address}</dd>
                  <dt className="text-ink-muted">Account ID</dt>
                  <dd className="numeric selectable break-all text-ink-body">
                    {account.account_id}
                  </dd>
                  <dt className="text-ink-muted">Scheme</dt>
                  <dd className="text-ink">Sr25519, as the ecosystem derives it</dd>
                </dl>
              </Panel>
            </>
          )}
        </section>
      </div>
    </div>
  );
}

function SendPanel({ from, onDone }: { from: string; onDone: () => void }) {
  const [to, setTo] = useState('');
  const [amount, setAmount] = useState('');
  const [preview, setPreview] = useState<Balance | null>(null);
  const [amountError, setAmountError] = useState<string | null>(null);
  const [confirming, setConfirming] = useState(false);
  const [sending, setSending] = useState(false);

  const notify = useQor((s) => s.notify);
  const refreshAllBalances = useQor((s) => s.refreshAllBalances);
  const token = useQor((s) => s.token);

  // Ask the host what the typed amount actually means, debounced.
  useEffect(() => {
    if (!amount.trim()) {
      setPreview(null);
      setAmountError(null);
      return;
    }
    const id = setTimeout(async () => {
      try {
        setPreview(await chain.parseAmount(amount));
        setAmountError(null);
      } catch (e) {
        setPreview(null);
        setAmountError(explain(e));
      }
    }, 250);
    return () => clearTimeout(id);
  }, [amount]);

  const recipientValid = looksLikeAddress(to);
  const ready = recipientValid && preview !== null && !amountError;

  const send = async () => {
    setSending(true);
    try {
      const receipt = await chain.send(from, to.trim(), amount);
      notify(
        'ok',
        `Sent ${receipt.amount_cgt} ${token?.symbol ?? 'CGT'} to ${shortAddress(receipt.to)}.`,
      );
      await refreshAllBalances();
      onDone();
    } catch (e) {
      notify('bad', explain(e));
      setConfirming(false);
    } finally {
      setSending(false);
    }
  };

  if (confirming && preview) {
    return (
      <Panel className="animate-rise p-6">
        <p className="eyebrow mb-4 text-accent">Confirm the transfer</p>

        <dl className="mb-6 grid grid-cols-[90px_1fr] gap-y-3 text-ui">
          <dt className="text-ink-muted">Amount</dt>
          <dd className="numeric text-title text-ink">
            {preview.cgt} {token?.symbol ?? 'CGT'}
          </dd>
          <dt className="text-ink-muted">From</dt>
          <dd className="numeric break-all text-ink-body">{from}</dd>
          <dt className="text-ink-muted">To</dt>
          <dd className="numeric break-all text-ink">{to.trim()}</dd>
        </dl>

        <p className="mb-5 border-l-2 border-warn bg-warn/5 px-3 py-2 text-caption text-warn">
          A transfer cannot be reversed. Check the recipient address character by
          character. The launcher then asks you to approve the signature in a system
          dialog.
        </p>

        <div className="flex gap-2">
          <button
            type="button"
            className="btn btn-ghost"
            disabled={sending}
            onClick={() => setConfirming(false)}
          >
            Back
          </button>
          <button
            type="button"
            className="btn btn-primary flex-1"
            disabled={sending}
            onClick={() => void send()}
          >
            {sending ? <Loader2 size={14} className="animate-spin" /> : <Check size={14} />}
            Sign and send
          </button>
        </div>
      </Panel>
    );
  }

  return (
    <Panel className="animate-rise p-6">
      <p className="eyebrow mb-5">Send {token?.symbol ?? 'CGT'}</p>

      <div className="mb-5 space-y-4">
        <Field
          label="Recipient address"
          hint={
            to && !recipientValid
              ? 'An address looks like 5GrwvaEF… . Hexadecimal account IDs are accepted too.'
              : undefined
          }
        >
          <input
            className={`field numeric ${to && !recipientValid ? 'field-invalid' : ''}`}
            placeholder="5…"
            autoComplete="off"
            spellCheck={false}
            value={to}
            onChange={(e) => setTo(e.target.value)}
          />
        </Field>

        <Field
          label={`Amount in ${token?.symbol ?? 'CGT'} or Sparks`}
          hint={
            amountError ??
            (preview
              ? `Sends exactly ${preview.display}${preview.in_sparks && preview.in_sparks !== preview.display ? ` (${preview.in_sparks})` : ''}`
              : 'Type CGT, or Sparks as "250 Sparks": 1 CGT is 1,000 Sparks')
          }
        >
          <input
            className={`field numeric ${amountError ? 'field-invalid' : ''}`}
            placeholder="0.00"
            inputMode="text"
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
          />
        </Field>
      </div>

      <div className="flex gap-2">
        <button type="button" className="btn btn-ghost" onClick={onDone}>
          Cancel
        </button>
        <button
          type="button"
          className="btn btn-primary flex-1"
          disabled={!ready}
          onClick={() => setConfirming(true)}
        >
          Review
          <ArrowUpRight size={13} />
        </button>
      </div>
    </Panel>
  );
}

function ReceivePanel({ address }: { address: string }) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(address);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <Panel className="animate-rise p-6">
      <p className="eyebrow mb-4">Receive CGT</p>
      <p className="mb-4 text-ui text-ink-muted">
        Anyone can send CGT to this address. Sharing it reveals your balance and
        transaction history, but never lets anyone spend from it.
      </p>

      <div className="mb-4 flex items-center gap-3 border border-edge bg-well p-4">
        <code className="numeric selectable flex-1 break-all text-ui text-ink">
          {address}
        </code>
      </div>

      <button type="button" className="btn" onClick={() => void copy()}>
        {copied ? <Check size={13} /> : <Copy size={13} />}
        {copied ? 'Copied' : 'Copy address'}
      </button>
    </Panel>
  );
}

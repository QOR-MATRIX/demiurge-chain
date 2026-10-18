/**
 * The CGT Vault.
 *
 * Holdings, accounts, send and receive. Two deliberate choices:
 *
 * - **The amount is parsed by the host, not here.** Typing in the send field
 *   round-trips to Rust, which returns the exact Spark value it would transfer.
 *   The preview the user confirms is therefore the number that gets signed, not
 *   a JavaScript approximation of it. `u128` does not fit in a `number`, and
 *   floating point has no business anywhere near a balance.
 * - **Sending is a two-stage act.** You compose, then you confirm against a
 *   summary showing the exact recipient and amount. A transfer cannot be undone,
 *   so a single click should not be able to cause one.
 * - **Whose account it is, said plainly** (0.1.12). The money belongs to an
 *   account's key, and a QOR ID is a name bound to one key. Each account is
 *   labelled with the QOR ID it belongs to, and when the account on screen is
 *   not the signed-in QOR ID's own, the page says so before anything is sent.
 */

import { useEffect, useState } from 'react';
import {
  ArrowDownLeft,
  ArrowUpRight,
  Check,
  Copy,
  Loader2,
  Plus,
  RefreshCw,
  TriangleAlert,
} from 'lucide-react';

import { chain, explain, looksLikeAddress, shortAddress, vault, type Balance, type Session } from '../lib/ipc';
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
  const refreshVault = useQor((s) => s.refreshVault);
  const notify = useQor((s) => s.notify);
  const session = useQor((s) => s.session);
  const owners = useQor((s) => s.owners);

  const [mode, setMode] = useState<Mode>('idle');
  const [refreshing, setRefreshing] = useState(false);

  const refresh = async () => {
    setRefreshing(true);
    await refreshAllBalances();
    setRefreshing(false);
  };

  const addAccount = async () => {
    try {
      await vault.addAccount('');
      await refreshVault();
      notify('ok', 'Account added.');
    } catch (e) {
      notify('bad', explain(e));
    }
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
            <span className="eyebrow">Accounts</span>
            <button
              type="button"
              className="btn btn-ghost px-2 py-1"
              title="Derive another account from this vault"
              onClick={() => void addAccount()}
            >
              <Plus size={13} />
            </button>
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
                    <span className="flex w-full items-center gap-2">
                      <span className="text-ui font-semibold">{a.label}</span>
                      {session?.address === a.address ? (
                        <span className="status-pill ml-auto text-ok" data-vault-owner={session.qor_id}>
                          {session.qor_id}
                        </span>
                      ) : owners[a.address] ? (
                        <span className="status-pill ml-auto text-ink-muted" data-vault-owner={owners[a.address]}>
                          {owners[a.address]}
                        </span>
                      ) : null}
                    </span>
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
              <Whose
                account={account.address}
                session={session}
                owner={owners[account.address] ?? null}
                held={accounts.some((a) => a.address === session?.address)}
                onSwitch={(address) => setActiveAddress(address)}
              />
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

/**
 * When the account on screen is not the signed-in QOR ID's own, say so, and say whose it is: sending from it spends
 * that account's CGT. A QOR ID whose account this PC holds can switch to it in one click.
 */
function Whose({
  account,
  session,
  owner,
  held,
  onSwitch,
}: {
  account: string;
  session: Session | null;
  owner: string | null;
  held: boolean;
  onSwitch: (address: string) => void;
}) {
  if (!session || session.address === account) return null;
  const whose = owner ? `${owner}'s` : 'not linked to any QOR ID this launcher has seen';
  return (
    <div className="mb-6 flex items-start gap-3 border-l-2 border-warn bg-warn/5 px-4 py-3" role="note" data-vault-mismatch>
      <TriangleAlert size={16} className="mt-0.5 flex-none text-warn" aria-hidden="true" />
      <div className="min-w-0 flex-1 text-ui text-ink-body">
        {session.address && held ? (
          <>
            <p>
              You are signed in as <strong className="text-ink">{session.qor_id}</strong>, but this account is {whose}.
              Sending from it spends this account&apos;s CGT, not {session.qor_id}&apos;s.
            </p>
            <button type="button" className="btn btn-primary mt-3" onClick={() => onSwitch(session.address!)} data-vault-switch>
              Show {session.qor_id}&apos;s account
            </button>
          </>
        ) : session.address ? (
          <p>
            You are signed in as <strong className="text-ink">{session.qor_id}</strong>, whose own account,{' '}
            <span className="numeric">{shortAddress(session.address, 10, 6)}</span>, is not in this PC&apos;s wallet. The
            account here is {whose}: sending from it spends its CGT, not {session.qor_id}&apos;s.
          </p>
        ) : (
          <p>
            You are signed in as <strong className="text-ink">{session.qor_id}</strong>, which has no account linked yet.
            The account here is {whose}.
          </p>
        )}
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
          label={`Amount in ${token?.symbol ?? 'CGT'}`}
          hint={
            amountError ??
            (preview
              ? `Sends exactly ${preview.display}`
              : token
                ? `Up to ${token.decimals} decimal places`
                : undefined)
          }
        >
          <input
            className={`field numeric ${amountError ? 'field-invalid' : ''}`}
            placeholder="0.00"
            inputMode="decimal"
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

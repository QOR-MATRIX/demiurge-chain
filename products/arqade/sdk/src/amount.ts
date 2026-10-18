// CGT amounts as integer embers (chain/runtime/src/denomination.rs). Never floating point (AGENTS.md section 5).
// Excess precision is refused, never rounded; anything past a u128 is refused.

export const DECIMALS = 18;
export const EMBERS_PER_CGT = BigInt(10) ** BigInt(DECIMALS);
export const U128_MAX = (BigInt(1) << BigInt(128)) - BigInt(1);
/** ADR-036. Read the runtime's own constant where a connection exists; this is the decided value it must match. */
export const EXISTENTIAL_DEPOSIT = BigInt(100) * EMBERS_PER_CGT;

const DECIMAL = /^(0|[1-9][0-9]*)(?:\.([0-9]+))?$/;

/** "12.5" CGT to embers. Refuses signs, exponents, separators, more than 18 decimals and values past u128. */
export function parseCgt(text: string): bigint {
  const m = DECIMAL.exec(text);
  if (!m) throw new RangeError(`Not a CGT amount: ${JSON.stringify(text)}`);
  const fraction = m[2] ?? '';
  if (fraction.length > DECIMALS) throw new RangeError(`More than ${DECIMALS} decimal places: ${text}`);
  const embers = BigInt(m[1]) * EMBERS_PER_CGT + BigInt(fraction.padEnd(DECIMALS, '0') || '0');
  return checkEmbers(embers);
}

/** Embers to the shortest exact CGT string: 12500000000000000000n is "12.5". */
export function formatCgt(embers: bigint): string {
  checkEmbers(embers);
  const whole = embers / EMBERS_PER_CGT;
  const fraction = (embers % EMBERS_PER_CGT).toString().padStart(DECIMALS, '0').replace(/0+$/, '');
  return fraction ? `${whole}.${fraction}` : whole.toString();
}

/** A embers value read from JSON, where amounts travel as decimal strings. */
export function embersFromJson(value: unknown): bigint {
  if (typeof value !== 'string' || !/^(0|[1-9][0-9]*)$/.test(value)) throw new RangeError('embers must be a decimal string');
  return checkEmbers(BigInt(value));
}

export function checkEmbers(embers: bigint): bigint {
  if (embers < BigInt(0) || embers > U128_MAX) throw new RangeError('Amount outside 0..u128');
  return embers;
}

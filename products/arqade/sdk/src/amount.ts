// CGT amounts as integer Sparks (chain/runtime/src/denomination.rs). Never floating point (AGENTS.md section 5).
// Excess precision is refused, never rounded; anything past a u128 is refused.

export const DECIMALS = 18;
export const SPARKS_PER_CGT = BigInt(10) ** BigInt(DECIMALS);
export const U128_MAX = (BigInt(1) << BigInt(128)) - BigInt(1);
/** ADR-036. Read the runtime's own constant where a connection exists; this is the decided value it must match. */
export const EXISTENTIAL_DEPOSIT = BigInt(100) * SPARKS_PER_CGT;

const DECIMAL = /^(0|[1-9][0-9]*)(?:\.([0-9]+))?$/;

/** "12.5" CGT to Sparks. Refuses signs, exponents, separators, more than 18 decimals and values past u128. */
export function parseCgt(text: string): bigint {
  const m = DECIMAL.exec(text);
  if (!m) throw new RangeError(`Not a CGT amount: ${JSON.stringify(text)}`);
  const fraction = m[2] ?? '';
  if (fraction.length > DECIMALS) throw new RangeError(`More than ${DECIMALS} decimal places: ${text}`);
  const sparks = BigInt(m[1]) * SPARKS_PER_CGT + BigInt(fraction.padEnd(DECIMALS, '0') || '0');
  return checkSparks(sparks);
}

/** Sparks to the shortest exact CGT string: 12500000000000000000n is "12.5". */
export function formatCgt(sparks: bigint): string {
  checkSparks(sparks);
  const whole = sparks / SPARKS_PER_CGT;
  const fraction = (sparks % SPARKS_PER_CGT).toString().padStart(DECIMALS, '0').replace(/0+$/, '');
  return fraction ? `${whole}.${fraction}` : whole.toString();
}

/** A Sparks value read from JSON, where amounts travel as decimal strings. */
export function sparksFromJson(value: unknown): bigint {
  if (typeof value !== 'string' || !/^(0|[1-9][0-9]*)$/.test(value)) throw new RangeError('Sparks must be a decimal string');
  return checkSparks(BigInt(value));
}

export function checkSparks(sparks: bigint): bigint {
  if (sparks < BigInt(0) || sparks > U128_MAX) throw new RangeError('Amount outside 0..u128');
  return sparks;
}

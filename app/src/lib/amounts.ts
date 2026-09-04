// Milestone 20: shared integer formatting keeps UI token math in base units.
export const TOKEN_DECIMALS = 6;
export const TOKEN_BASE_UNITS = 1_000_000n;

export function formatBaseUnits(amount: bigint, decimals = TOKEN_DECIMALS): string {
  if (decimals < 0 || !Number.isInteger(decimals)) {
    throw new Error("decimals must be a non-negative integer");
  }

  const sign = amount < 0n ? "-" : "";
  const absolute = amount < 0n ? -amount : amount;
  const scale = 10n ** BigInt(decimals);
  const whole = absolute / scale;
  const fraction = absolute % scale;

  if (fraction === 0n || decimals === 0) {
    return `${sign}${whole.toString()}`;
  }

  const padded = fraction.toString().padStart(decimals, "0").replace(/0+$/, "");
  return `${sign}${whole.toString()}.${padded}`;
}

export function parseBaseUnits(value: string, decimals = TOKEN_DECIMALS): bigint {
  if (decimals < 0 || !Number.isInteger(decimals)) {
    throw new Error("decimals must be a non-negative integer");
  }

  const normalized = value.trim();
  if (!/^(0|[1-9]\d*)(\.\d+)?$/.test(normalized)) {
    throw new Error("amount must be a positive decimal string");
  }

  const [whole, fraction = ""] = normalized.split(".");
  if (fraction.length > decimals) {
    throw new Error(`amount has more than ${decimals} decimal places`);
  }

  const scale = 10n ** BigInt(decimals);
  const paddedFraction = fraction.padEnd(decimals, "0");
  return BigInt(whole) * scale + BigInt(paddedFraction || "0");
}

export function requirePositiveBaseUnits(amount: bigint): bigint {
  if (amount <= 0n) {
    throw new Error("amount must be greater than zero");
  }

  return amount;
}

export function toU64Amount(amount: bigint): bigint {
  requirePositiveBaseUnits(amount);

  if (amount > 18_446_744_073_709_551_615n) {
    throw new Error("amount exceeds u64 maximum");
  }

  return amount;
}

export function shortAddress(value: string, visible = 4): string {
  if (value.length <= visible * 2 + 3) {
    return value;
  }

  return `${value.slice(0, visible)}...${value.slice(-visible)}`;
}

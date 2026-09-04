import { describe, expect, it } from "vitest";

import { formatBaseUnits, parseBaseUnits, shortAddress, toU64Amount, TOKEN_BASE_UNITS } from "@/lib/amounts";

describe("amount formatting", () => {
  it("formats six-decimal token base units without floating point", () => {
    expect(formatBaseUnits(1_234_567n)).toBe("1.234567");
    expect(formatBaseUnits(1_200_000n)).toBe("1.2");
    expect(formatBaseUnits(1_000n)).toBe("0.001");
    expect(formatBaseUnits(TOKEN_BASE_UNITS * 1_000n)).toBe("1000");
  });

  it("parses decimal display strings into integer base units", () => {
    expect(parseBaseUnits("1")).toBe(1_000_000n);
    expect(parseBaseUnits("1.234567")).toBe(1_234_567n);
    expect(parseBaseUnits("0.000001")).toBe(1n);
  });

  it("rejects precision that cannot exist on chain", () => {
    expect(() => parseBaseUnits("0.0000001")).toThrow("decimal places");
    expect(() => parseBaseUnits("-1")).toThrow("positive decimal");
    expect(() => parseBaseUnits("1e6")).toThrow("positive decimal");
    expect(() => parseBaseUnits("")).toThrow("positive decimal");
  });

  it("rejects non-positive and oversized transaction amounts", () => {
    expect(() => toU64Amount(0n)).toThrow("greater than zero");
    expect(() => toU64Amount(18_446_744_073_709_551_616n)).toThrow("u64");
  });
});

describe("address formatting", () => {
  it("shortens long addresses", () => {
    expect(shortAddress("123456789ABCDEFG", 4)).toBe("1234...DEFG");
  });
});

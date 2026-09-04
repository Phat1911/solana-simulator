import { describe, expect, it } from "vitest";

import { buildUserTransactionMessage } from "@/lib/transaction-messages";
import { buildStakeTransaction, type UserActionContext } from "@/lib/user-instructions";

const context: UserActionContext = {
  user: "11111111111111111111111111111111",
  stakingProgram: "8Dkwd74ntycfAMWKeudjELGroj5pqUpWk4MyLijuf1W7",
  demoFaucetProgram: "J12YAqC7dWbVWVAFveRdJ8SJ3sYc6roP3WJekhy4bDkM",
  pool: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
  stakeMint: "So11111111111111111111111111111111111111112",
  rewardMint: "SysvarRent111111111111111111111111111111111",
  stakeVault: "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
  rewardVault: "SysvarC1ock11111111111111111111111111111111",
};

describe("transaction message assembly", () => {
  it("keeps prepared user instructions in order", async () => {
    const plan = await buildStakeTransaction(context, 1_000_000n, { includeOpenPosition: true });
    const message = buildUserTransactionMessage(plan, context.user, {
      blockhash: "EETubP5AKHgjPAhzPAFcb8BAY1hMH639CWCFTqi3hq1k" as never,
      lastValidBlockHeight: 123n,
    });

    expect(message.instructions).toHaveLength(2);
    expect(message.feePayer.address.toString()).toBe(context.user);
  });
});

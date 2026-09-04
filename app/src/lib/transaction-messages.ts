import {
  address,
  appendTransactionMessageInstructions,
  createTransactionMessage,
  setTransactionMessageFeePayer,
  setTransactionMessageLifetimeUsingBlockhash,
} from "@solana/kit";

import { type PreparedUserTransaction } from "@/lib/user-instructions";

export type LatestBlockhash = Parameters<typeof setTransactionMessageLifetimeUsingBlockhash>[0];

// Milestone 21: assemble prepared user instructions into a Solana transaction message.
export function buildUserTransactionMessage(plan: PreparedUserTransaction, feePayer: string, latestBlockhash: LatestBlockhash) {
  const message = createTransactionMessage({ version: 0 });
  const withPayer = setTransactionMessageFeePayer(address(feePayer), message);
  const withLifetime = setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, withPayer);

  return appendTransactionMessageInstructions([...plan.instructions], withLifetime);
}

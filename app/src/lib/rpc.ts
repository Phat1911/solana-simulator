import { address, createSolanaRpc, devnet } from "@solana/kit";

import {
  decodePoolState,
  decodePositionState,
  decodeProposalState,
  type PoolState,
  type PositionState,
  type ProposalState,
} from "@/lib/account-decoders";

export type AccountReadState =
  | { status: "not-configured" }
  | { status: "found"; lamports: string; owner?: string; executable: boolean }
  | { status: "missing" }
  | { status: "error"; message: string };

function endpointForRpc(endpoint: string) {
  return endpoint.includes("devnet") ? devnet(endpoint) : endpoint;
}

function decodeBase64(value: string): Uint8Array {
  if (typeof Buffer !== "undefined") {
    return new Uint8Array(Buffer.from(value, "base64"));
  }

  const binary = atob(value);
  return Uint8Array.from(binary, (char) => char.charCodeAt(0));
}

function getBase64Data(data: unknown): string | undefined {
  if (Array.isArray(data) && typeof data[0] === "string") {
    return data[0];
  }

  return undefined;
}

// Milestone 20: read-only RPC plumbing for frontend account discovery.
export async function readAccount(endpoint: string, accountAddress?: string): Promise<AccountReadState> {
  if (!accountAddress) {
    return { status: "not-configured" };
  }

  try {
    const rpc = createSolanaRpc(endpointForRpc(endpoint));
    const response = (await rpc.getAccountInfo(address(accountAddress), { encoding: "base64" }).send()) as {
      value: null | {
        lamports: bigint | number | string;
        owner?: string;
        executable?: boolean;
      };
    };

    if (!response.value) {
      return { status: "missing" };
    }

    return {
      status: "found",
      lamports: response.value.lamports.toString(),
      owner: response.value.owner,
      executable: Boolean(response.value.executable),
    };
  } catch (error) {
    return {
      status: "error",
      message: error instanceof Error ? error.message : "Unknown RPC error",
    };
  }
}

async function readAccountBytes(endpoint: string, accountAddress?: string): Promise<Uint8Array | undefined> {
  if (!accountAddress) {
    return undefined;
  }

  const rpc = createSolanaRpc(endpointForRpc(endpoint));
  const response = (await rpc.getAccountInfo(address(accountAddress), { encoding: "base64" }).send()) as {
    value: null | { data?: unknown };
  };
  const encoded = getBase64Data(response.value?.data);
  return encoded ? decodeBase64(encoded) : undefined;
}

// Milestone 21: decoded reads feed user balances and reward estimates without trusting them on chain.
export async function readPoolState(endpoint: string, pool?: string): Promise<PoolState | undefined> {
  const bytes = await readAccountBytes(endpoint, pool);
  return bytes ? decodePoolState(bytes) : undefined;
}

export async function readPositionState(endpoint: string, position?: string): Promise<PositionState | undefined> {
  const bytes = await readAccountBytes(endpoint, position);
  return bytes ? decodePositionState(bytes) : undefined;
}

export async function readProposalState(endpoint: string, proposal?: string): Promise<ProposalState | undefined> {
  const bytes = await readAccountBytes(endpoint, proposal);
  return bytes ? decodeProposalState(bytes) : undefined;
}

export async function readTokenBalance(endpoint: string, tokenAccount?: string): Promise<bigint | undefined> {
  if (!tokenAccount) {
    return undefined;
  }

  try {
    const rpc = createSolanaRpc(endpointForRpc(endpoint));
    const response = (await rpc.getTokenAccountBalance(address(tokenAccount)).send()) as {
      value?: { amount?: string };
    };
    return response.value?.amount ? BigInt(response.value.amount) : undefined;
  } catch {
    return undefined;
  }
}

import { address, createSolanaRpc, devnet } from "@solana/kit";

export type AccountReadState =
  | { status: "not-configured" }
  | { status: "found"; lamports: string; owner?: string; executable: boolean }
  | { status: "missing" }
  | { status: "error"; message: string };

function endpointForRpc(endpoint: string) {
  return endpoint.includes("devnet") ? devnet(endpoint) : endpoint;
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

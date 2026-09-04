"use client";

import { Activity, Database, ExternalLink, Network, ShieldCheck } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { UserActions } from "@/components/user-actions";
import { WalletPanel } from "@/components/wallet-panel";
import { formatBaseUnits, shortAddress } from "@/lib/amounts";
import { estimatePendingRewardScaled, formatScaledReward, type PoolState, type PositionState } from "@/lib/account-decoders";
import { type DeploymentConfig } from "@/lib/deployment";
import { type AccountReadState, readAccount } from "@/lib/rpc";
import { deriveAssociatedTokenAccount, derivePoolAuthorityPda, derivePositionPda } from "@/lib/pdas";
import { readPoolState, readPositionState, readTokenBalance } from "@/lib/rpc";
import { type UserActionContext } from "@/lib/user-instructions";

function statusLabel(account: AccountReadState): string {
  switch (account.status) {
    case "found":
      return "Found";
    case "missing":
      return "Missing";
    case "not-configured":
      return "Not configured";
    case "error":
      return "RPC error";
  }
}

function explorerHref(cluster: DeploymentConfig["cluster"], value?: string): string | undefined {
  if (!value) {
    return undefined;
  }

  const clusterParam = cluster === "devnet" ? "?cluster=devnet" : "?cluster=custom";
  return `https://explorer.solana.com/address/${value}${clusterParam}`;
}

function AddressRow({ label, value, cluster }: { label: string; value?: string; cluster: DeploymentConfig["cluster"] }) {
  const href = explorerHref(cluster, value);

  return (
    <div className="address-row">
      <span>{label}</span>
      <strong>{value ? shortAddress(value, 6) : "Not configured"}</strong>
      {href ? (
        <a aria-label={`${label} on Explorer`} href={href} rel="noreferrer" target="_blank">
          <ExternalLink size={15} />
        </a>
      ) : null}
    </div>
  );
}

export function Dashboard({ initialDeployment }: { initialDeployment: DeploymentConfig }) {
  const deployment = useMemo(() => initialDeployment, [initialDeployment]);
  const [connectedAccount, setConnectedAccount] = useState<string>();
  const [poolAccount, setPoolAccount] = useState<AccountReadState>({ status: "not-configured" });
  const [poolState, setPoolState] = useState<PoolState>();
  const [positionState, setPositionState] = useState<PositionState>();
  const [derivedPoolAuthority, setDerivedPoolAuthority] = useState<string>();
  const [userStakeAta, setUserStakeAta] = useState<string>();
  const [userRewardAta, setUserRewardAta] = useState<string>();
  const [stakeBalance, setStakeBalance] = useState<bigint>();
  const [rewardBalance, setRewardBalance] = useState<bigint>();
  const [isRefreshing, setRefreshing] = useState(false);

  const resolved = {
    pool: deployment.pool,
    poolAuthority: deployment.poolAuthority ?? derivedPoolAuthority,
    stakeMint: poolState?.stakeMint ?? deployment.stakeMint,
    rewardMint: poolState?.rewardMint ?? deployment.rewardMint,
    stakeVault: poolState?.stakeVault ?? deployment.stakeVault,
    rewardVault: poolState?.rewardVault ?? deployment.rewardVault,
  };

  const userActionContext: UserActionContext | undefined =
    connectedAccount &&
    deployment.stakingProgram &&
    deployment.demoFaucetProgram &&
    resolved.pool &&
    resolved.stakeMint &&
    resolved.rewardMint &&
    resolved.stakeVault &&
    resolved.rewardVault
      ? {
          user: connectedAccount,
          stakingProgram: deployment.stakingProgram,
          demoFaucetProgram: deployment.demoFaucetProgram,
          pool: resolved.pool,
          stakeMint: resolved.stakeMint,
          rewardMint: resolved.rewardMint,
          stakeVault: resolved.stakeVault,
          rewardVault: resolved.rewardVault,
        }
      : undefined;

  const pendingRewardScaled =
    poolState && positionState ? estimatePendingRewardScaled(poolState, positionState) : undefined;

  async function refreshPool() {
    setRefreshing(true);
    setPoolAccount(await readAccount(deployment.endpoint, deployment.pool));
    setPoolState(await readPoolState(deployment.endpoint, deployment.pool));
    setRefreshing(false);
  }

  async function refreshUser() {
    if (!connectedAccount || !deployment.stakingProgram || !resolved.pool || !resolved.stakeMint || !resolved.rewardMint) {
      setPositionState(undefined);
      setUserStakeAta(undefined);
      setUserRewardAta(undefined);
      setStakeBalance(undefined);
      setRewardBalance(undefined);
      return;
    }

    const [[positionAddress], [poolAuthority], [stakeAta], [rewardAta]] = await Promise.all([
      derivePositionPda(deployment.stakingProgram, resolved.pool, connectedAccount),
      derivePoolAuthorityPda(deployment.stakingProgram, resolved.pool),
      deriveAssociatedTokenAccount(connectedAccount, resolved.stakeMint),
      deriveAssociatedTokenAccount(connectedAccount, resolved.rewardMint),
    ]);

    setDerivedPoolAuthority(poolAuthority.toString());

    const stakeAtaAddress = stakeAta.toString();
    const rewardAtaAddress = rewardAta.toString();
    setUserStakeAta(stakeAtaAddress);
    setUserRewardAta(rewardAtaAddress);

    const [nextPositionState, nextStakeBalance, nextRewardBalance] = await Promise.all([
      readPositionState(deployment.endpoint, positionAddress.toString()),
      readTokenBalance(deployment.endpoint, stakeAtaAddress),
      readTokenBalance(deployment.endpoint, rewardAtaAddress),
    ]);

    setPositionState(nextPositionState);
    setStakeBalance(nextStakeBalance);
    setRewardBalance(nextRewardBalance);
  }

  useEffect(() => {
    void refreshPool();
  }, []);

  useEffect(() => {
    void refreshUser();
  }, [connectedAccount, poolState]);

  return (
    <main className="app-shell">
      <header className="topbar">
        <div>
          <p className="eyebrow">Slot Staking</p>
          <h1>Devnet Console</h1>
        </div>
        <div className="network-pill">
          <Network size={16} />
          <span>{deployment.cluster}</span>
        </div>
      </header>

      <section className="status-strip">
        <div className="metric">
          <Activity size={18} />
          <span>RPC</span>
          <strong>{deployment.endpoint}</strong>
        </div>
        <div className="metric">
          <Database size={18} />
          <span>Pool</span>
          <strong>{statusLabel(poolAccount)}</strong>
        </div>
        <div className="metric">
          <ShieldCheck size={18} />
          <span>Admins</span>
          <strong>{deployment.admins.length}/3</strong>
        </div>
      </section>

      <div className="content-grid">
        <section className="panel">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Deployment</p>
              <h2>Known Accounts</h2>
            </div>
            <button className="text-button" disabled={isRefreshing} onClick={() => void refreshPool()} type="button">
              {isRefreshing ? "Reading" : "Refresh"}
            </button>
          </div>

          <div className="address-list">
            <AddressRow cluster={deployment.cluster} label="Staking Program" value={deployment.stakingProgram} />
            <AddressRow cluster={deployment.cluster} label="Demo Faucet" value={deployment.demoFaucetProgram} />
            <AddressRow cluster={deployment.cluster} label="Pool" value={resolved.pool} />
            <AddressRow cluster={deployment.cluster} label="Pool Authority" value={resolved.poolAuthority} />
            <AddressRow cluster={deployment.cluster} label="STAKE Mint" value={resolved.stakeMint} />
            <AddressRow cluster={deployment.cluster} label="REWARD Mint" value={resolved.rewardMint} />
            <AddressRow cluster={deployment.cluster} label="Stake Vault" value={resolved.stakeVault} />
            <AddressRow cluster={deployment.cluster} label="Reward Vault" value={resolved.rewardVault} />
          </div>
        </section>

        <WalletPanel onAccountChange={setConnectedAccount} />
      </div>

      <section className="panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">Account Read</p>
            <h2>Pool State</h2>
          </div>
        </div>
        <dl className="detail-list horizontal">
          <div>
            <dt>Status</dt>
            <dd>{statusLabel(poolAccount)}</dd>
          </div>
          <div>
            <dt>Lamports</dt>
            <dd>{poolAccount.status === "found" ? poolAccount.lamports : "0"}</dd>
          </div>
          <div>
            <dt>Executable</dt>
            <dd>{poolAccount.status === "found" && poolAccount.executable ? "Yes" : "No"}</dd>
          </div>
          <div>
            <dt>Paused</dt>
            <dd>{poolState?.paused ? "Yes" : "No"}</dd>
          </div>
          <div>
            <dt>Total Staked</dt>
            <dd>{poolState ? formatBaseUnits(poolState.totalStaked) : "0"}</dd>
          </div>
          <div>
            <dt>Reward Rate</dt>
            <dd>{poolState ? formatBaseUnits(poolState.rewardRatePerSlot) : "0"}</dd>
          </div>
        </dl>
        {poolAccount.status === "error" ? <p className="error-text">{poolAccount.message}</p> : null}
      </section>

      <section className="panel">
        <div className="panel-heading">
          <div>
            <p className="eyebrow">Position</p>
            <h2>User State</h2>
          </div>
          <button className="text-button" onClick={() => void refreshUser()} type="button">
            Refresh
          </button>
        </div>
        <dl className="detail-list horizontal">
          <div>
            <dt>Position</dt>
            <dd>{positionState ? "Found" : "Missing"}</dd>
          </div>
          <div>
            <dt>User STAKE ATA</dt>
            <dd>{userStakeAta ? shortAddress(userStakeAta, 6) : "None"}</dd>
          </div>
          <div>
            <dt>User REWARD ATA</dt>
            <dd>{userRewardAta ? shortAddress(userRewardAta, 6) : "None"}</dd>
          </div>
          <div>
            <dt>Principal</dt>
            <dd>{positionState ? formatBaseUnits(positionState.stakedAmount) : "0"}</dd>
          </div>
          <div>
            <dt>Estimated Pending</dt>
            <dd>{pendingRewardScaled === undefined ? "0" : formatScaledReward(pendingRewardScaled)}</dd>
          </div>
        </dl>
      </section>

      <UserActions
        context={userActionContext}
        pendingRewardScaled={pendingRewardScaled}
        poolPaused={poolState?.paused}
        positionExists={Boolean(positionState)}
        rewardBalance={rewardBalance}
        stakeBalance={stakeBalance}
        stakedAmount={positionState?.stakedAmount}
      />
    </main>
  );
}

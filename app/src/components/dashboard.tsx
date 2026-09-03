"use client";

import { Activity, Database, ExternalLink, Network, ShieldCheck } from "lucide-react";
import { useEffect, useMemo, useState } from "react";

import { WalletPanel } from "@/components/wallet-panel";
import { shortAddress } from "@/lib/amounts";
import { type DeploymentConfig } from "@/lib/deployment";
import { type AccountReadState, readAccount } from "@/lib/rpc";

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
  const [poolAccount, setPoolAccount] = useState<AccountReadState>({ status: "not-configured" });
  const [isRefreshing, setRefreshing] = useState(false);

  async function refreshPool() {
    setRefreshing(true);
    setPoolAccount(await readAccount(deployment.endpoint, deployment.pool));
    setRefreshing(false);
  }

  useEffect(() => {
    void refreshPool();
  }, []);

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
            <AddressRow cluster={deployment.cluster} label="Pool" value={deployment.pool} />
            <AddressRow cluster={deployment.cluster} label="Pool Authority" value={deployment.poolAuthority} />
            <AddressRow cluster={deployment.cluster} label="STAKE Mint" value={deployment.stakeMint} />
            <AddressRow cluster={deployment.cluster} label="REWARD Mint" value={deployment.rewardMint} />
            <AddressRow cluster={deployment.cluster} label="Stake Vault" value={deployment.stakeVault} />
            <AddressRow cluster={deployment.cluster} label="Reward Vault" value={deployment.rewardVault} />
          </div>
        </section>

        <WalletPanel />
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
        </dl>
        {poolAccount.status === "error" ? <p className="error-text">{poolAccount.message}</p> : null}
      </section>
    </main>
  );
}

"use client";

import { RefreshCw, Wallet } from "lucide-react";
import { useEffect, useState } from "react";

import { connectWallet, getDetectedWallets, subscribeToWalletChanges, type WalletLike } from "@/lib/wallets";
import { shortAddress } from "@/lib/amounts";

export function WalletPanel() {
  const [wallets, setWallets] = useState<readonly WalletLike[]>([]);
  const [selected, setSelected] = useState<string>();
  const [account, setAccount] = useState<string>();
  const [error, setError] = useState<string>();

  function refresh() {
    setWallets(getDetectedWallets());
  }

  useEffect(() => {
    refresh();
    return subscribeToWalletChanges(refresh);
  }, []);

  async function onConnect(wallet: WalletLike) {
    setError(undefined);
    setSelected(wallet.name);

    try {
      const accounts = await connectWallet(wallet);
      setAccount(accounts[0]?.address);
    } catch (caught) {
      setError(caught instanceof Error ? caught.message : "Wallet connection failed");
    }
  }

  return (
    <section className="panel wallet-panel">
      <div className="panel-heading">
        <div>
          <p className="eyebrow">Wallet</p>
          <h2>Connection</h2>
        </div>
        <button aria-label="Refresh wallets" className="icon-button" onClick={refresh} type="button">
          <RefreshCw size={18} />
        </button>
      </div>

      <div className="wallet-list">
        {wallets.length === 0 ? (
          <div className="empty-row">No Wallet Standard adapter detected</div>
        ) : (
          wallets.map((wallet, index) => (
            <button
              className={wallet.name === selected ? "wallet-row selected" : "wallet-row"}
              key={`${wallet.name}-${index}`}
              onClick={() => void onConnect(wallet)}
              type="button"
            >
              {wallet.icon ? <img alt="" className="wallet-icon" src={wallet.icon} /> : <Wallet size={18} />}
              <span>{wallet.name}</span>
            </button>
          ))
        )}
      </div>

      <dl className="detail-list">
        <div>
          <dt>Status</dt>
          <dd>{account ? "Connected" : "Disconnected"}</dd>
        </div>
        <div>
          <dt>Account</dt>
          <dd>{account ? shortAddress(account, 6) : "None"}</dd>
        </div>
      </dl>

      {error ? <p className="error-text">{error}</p> : null}
    </section>
  );
}

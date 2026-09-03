/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/demo_faucet.json`.
 */
export type DemoFaucet = {
  "address": "J12YAqC7dWbVWVAFveRdJ8SJ3sYc6roP3WJekhy4bDkM",
  "metadata": {
    "name": "demoFaucet",
    "version": "0.1.0",
    "spec": "0.1.0",
    "repository": "https://github.com/example/slot-staking"
  },
  "instructions": [
    {
      "name": "claimTestStake",
      "docs": [
        "Milestone 17: mint exactly 1,000 STAKE once to the claimant's canonical ATA."
      ],
      "discriminator": [
        160,
        5,
        137,
        252,
        241,
        44,
        153,
        233
      ],
      "accounts": [
        {
          "name": "claimant",
          "writable": true,
          "signer": true
        },
        {
          "name": "stakeMint",
          "writable": true
        },
        {
          "name": "faucetAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  97,
                  117,
                  99,
                  101,
                  116,
                  45,
                  97,
                  117,
                  116,
                  104,
                  111,
                  114,
                  105,
                  116,
                  121
                ]
              },
              {
                "kind": "account",
                "path": "stakeMint"
              }
            ]
          }
        },
        {
          "name": "claimantStakeAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "claimant"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "stakeMint"
              }
            ],
            "program": {
              "kind": "const",
              "value": [
                140,
                151,
                37,
                143,
                78,
                36,
                137,
                241,
                187,
                61,
                16,
                41,
                20,
                142,
                13,
                131,
                11,
                90,
                19,
                153,
                218,
                255,
                16,
                132,
                4,
                142,
                123,
                216,
                219,
                233,
                248,
                89
              ]
            }
          }
        },
        {
          "name": "claimReceipt",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  102,
                  97,
                  117,
                  99,
                  101,
                  116,
                  45,
                  99,
                  108,
                  97,
                  105,
                  109
                ]
              },
              {
                "kind": "account",
                "path": "stakeMint"
              },
              {
                "kind": "account",
                "path": "claimant"
              }
            ]
          }
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        },
        {
          "name": "associatedTokenProgram",
          "address": "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    }
  ],
  "accounts": [
    {
      "name": "faucetClaimReceipt",
      "discriminator": [
        62,
        242,
        190,
        160,
        119,
        99,
        217,
        250
      ]
    }
  ],
  "events": [
    {
      "name": "faucetClaimed",
      "discriminator": [
        153,
        213,
        25,
        224,
        176,
        249,
        203,
        218
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "invalidTokenDecimals",
      "msg": "Token mint must use exactly six decimals"
    },
    {
      "code": 6001,
      "name": "invalidMintAuthority",
      "msg": "Stake mint authority must be the Faucet Authority PDA"
    },
    {
      "code": 6002,
      "name": "invalidTokenProgram",
      "msg": "Only the original SPL Token Program is supported"
    },
    {
      "code": 6003,
      "name": "faucetAlreadyClaimed",
      "msg": "Wallet has already claimed test stake for this mint"
    }
  ],
  "types": [
    {
      "name": "faucetClaimReceipt",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "type": "u8"
          },
          {
            "name": "stakeMint",
            "type": "pubkey"
          },
          {
            "name": "claimant",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "claimedSlot",
            "type": "u64"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "faucetClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "stakeMint",
            "type": "pubkey"
          },
          {
            "name": "claimant",
            "type": "pubkey"
          },
          {
            "name": "claimReceipt",
            "type": "pubkey"
          },
          {
            "name": "destination",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    }
  ]
};

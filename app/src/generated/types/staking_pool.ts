/**
 * Program IDL in camelCase format in order to be used in JS/TS.
 *
 * Note that this is only a type helper and is not the actual IDL. The original
 * IDL can be found at `target/idl/staking_pool.json`.
 */
export type StakingPool = {
  "address": "Fg6PaFpoGXkYsidMpWxTWqkFrnDRBTTnyW6m9n6eGJZ",
  "metadata": {
    "name": "stakingPool",
    "version": "0.1.0",
    "spec": "0.1.0",
    "repository": "https://github.com/example/slot-staking"
  },
  "instructions": [
    {
      "name": "approveProposal",
      "docs": [
        "Milestone 15: add one distinct current-admin approval to a proposal."
      ],
      "discriminator": [
        136,
        108,
        102,
        85,
        98,
        114,
        7,
        147
      ],
      "accounts": [
        {
          "name": "admin",
          "signer": true
        },
        {
          "name": "pool"
        },
        {
          "name": "proposal",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  112,
                  111,
                  115,
                  97,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "proposal.proposal_id",
                "account": "proposal"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "claimRewards",
      "docs": [
        "Milestone 12: claim whole REWARD base units, preserving scaled remainder."
      ],
      "discriminator": [
        4,
        144,
        132,
        71,
        116,
        23,
        151,
        80
      ],
      "accounts": [
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "poolAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108,
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
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "rewardMint"
        },
        {
          "name": "rewardVault",
          "writable": true
        },
        {
          "name": "userRewardAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
              },
              {
                "kind": "account",
                "path": "rewardMint"
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
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": []
    },
    {
      "name": "closePosition",
      "docs": [
        "Milestone 8: close only an empty, reward-free position owned by signer."
      ],
      "discriminator": [
        123,
        134,
        81,
        0,
        49,
        68,
        98,
        98
      ],
      "accounts": [
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool"
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "closeProposal",
      "docs": [
        "Milestone 16: close only executed, expired, or stale proposals to creator."
      ],
      "discriminator": [
        213,
        178,
        139,
        19,
        50,
        191,
        82,
        245
      ],
      "accounts": [
        {
          "name": "payer",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool"
        },
        {
          "name": "proposal",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  112,
                  111,
                  115,
                  97,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "proposal.proposal_id",
                "account": "proposal"
              }
            ]
          }
        },
        {
          "name": "creator",
          "writable": true
        }
      ],
      "args": []
    },
    {
      "name": "createProposal",
      "docs": [
        "Milestone 15: create one immutable allowlisted admin proposal."
      ],
      "discriminator": [
        132,
        116,
        68,
        174,
        216,
        160,
        198,
        22
      ],
      "accounts": [
        {
          "name": "creator",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "proposal",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  112,
                  111,
                  115,
                  97,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "arg",
                "path": "proposalId"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "proposalId",
          "type": "u64"
        },
        {
          "name": "action",
          "type": {
            "defined": {
              "name": "proposalAction"
            }
          }
        }
      ]
    },
    {
      "name": "emergencyWithdraw",
      "docs": [
        "Milestone 14: return all principal and recycle the user's reward entitlement."
      ],
      "discriminator": [
        239,
        45,
        203,
        64,
        150,
        73,
        218,
        92
      ],
      "accounts": [
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "poolAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108,
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
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "stakeMint"
        },
        {
          "name": "rewardMint"
        },
        {
          "name": "userStakeAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
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
          "name": "stakeVault",
          "writable": true
        },
        {
          "name": "rewardVault",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": []
    },
    {
      "name": "executeProposal",
      "docs": [
        "Milestone 16: execute exactly one approved proposal action once."
      ],
      "discriminator": [
        186,
        60,
        116,
        133,
        108,
        128,
        111,
        28
      ],
      "accounts": [
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "proposal",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  114,
                  111,
                  112,
                  111,
                  115,
                  97,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "proposal.proposal_id",
                "account": "proposal"
              }
            ]
          }
        }
      ],
      "args": []
    },
    {
      "name": "fundRewards",
      "docs": [
        "Milestone 9: checkpoint, transfer real REWARD tokens, then credit budget."
      ],
      "discriminator": [
        114,
        64,
        163,
        112,
        175,
        167,
        19,
        121
      ],
      "accounts": [
        {
          "name": "sourceAuthority",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "sourceRewardAccount",
          "writable": true
        },
        {
          "name": "rewardMint"
        },
        {
          "name": "rewardVault",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "initializePool",
      "docs": [
        "Milestone 7: create the canonical pool account and PDA-owned vault ATAs."
      ],
      "discriminator": [
        95,
        180,
        10,
        172,
        84,
        174,
        232,
        40
      ],
      "accounts": [
        {
          "name": "initializer",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108
                ]
              },
              {
                "kind": "account",
                "path": "initializer"
              },
              {
                "kind": "arg",
                "path": "poolId"
              }
            ]
          }
        },
        {
          "name": "poolAuthority",
          "docs": [
            "canonical seeds here and it only acts as the vault ATA authority."
          ],
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108,
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
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "stakeMint"
        },
        {
          "name": "rewardMint"
        },
        {
          "name": "stakeVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "poolAuthority"
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
          "name": "rewardVault",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "poolAuthority"
              },
              {
                "kind": "account",
                "path": "tokenProgram"
              },
              {
                "kind": "account",
                "path": "rewardMint"
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
        },
        {
          "name": "rent",
          "address": "SysvarRent111111111111111111111111111111111"
        }
      ],
      "args": [
        {
          "name": "poolId",
          "type": "u64"
        },
        {
          "name": "admins",
          "type": {
            "array": [
              "pubkey",
              3
            ]
          }
        },
        {
          "name": "maxRewardRatePerSlot",
          "type": "u64"
        }
      ]
    },
    {
      "name": "openPosition",
      "docs": [
        "Milestone 8: create one canonical empty position for a user in a pool."
      ],
      "discriminator": [
        135,
        128,
        47,
        77,
        15,
        152,
        240,
        49
      ],
      "accounts": [
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool"
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "systemProgram",
          "address": "11111111111111111111111111111111"
        }
      ],
      "args": []
    },
    {
      "name": "pausePool",
      "docs": [
        "Milestone 13: any current admin can checkpoint and immediately pause."
      ],
      "discriminator": [
        160,
        15,
        12,
        189,
        160,
        0,
        243,
        245
      ],
      "accounts": [
        {
          "name": "admin",
          "signer": true
        },
        {
          "name": "pool",
          "writable": true
        }
      ],
      "args": []
    },
    {
      "name": "stake",
      "docs": [
        "Milestone 10: stake canonical STAKE ATA principal after checkpoint and settlement."
      ],
      "discriminator": [
        206,
        176,
        202,
        18,
        200,
        209,
        179,
        108
      ],
      "accounts": [
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "stakeMint"
        },
        {
          "name": "rewardMint"
        },
        {
          "name": "userStakeAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
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
          "name": "stakeVault",
          "writable": true
        },
        {
          "name": "rewardVault",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    },
    {
      "name": "unstake",
      "docs": [
        "Milestone 11: unstake principal only, preserving settled rewards."
      ],
      "discriminator": [
        90,
        95,
        107,
        42,
        205,
        124,
        50,
        225
      ],
      "accounts": [
        {
          "name": "user",
          "writable": true,
          "signer": true
        },
        {
          "name": "pool",
          "writable": true
        },
        {
          "name": "poolAuthority",
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  111,
                  108,
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
                "path": "pool"
              }
            ]
          }
        },
        {
          "name": "position",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "const",
                "value": [
                  112,
                  111,
                  115,
                  105,
                  116,
                  105,
                  111,
                  110
                ]
              },
              {
                "kind": "account",
                "path": "pool"
              },
              {
                "kind": "account",
                "path": "user"
              }
            ]
          }
        },
        {
          "name": "stakeMint"
        },
        {
          "name": "rewardMint"
        },
        {
          "name": "userStakeAccount",
          "writable": true,
          "pda": {
            "seeds": [
              {
                "kind": "account",
                "path": "user"
              },
              {
                "kind": "const",
                "value": [
                  6,
                  221,
                  246,
                  225,
                  215,
                  101,
                  161,
                  147,
                  217,
                  203,
                  225,
                  70,
                  206,
                  235,
                  121,
                  172,
                  28,
                  180,
                  133,
                  237,
                  95,
                  91,
                  55,
                  145,
                  58,
                  140,
                  245,
                  133,
                  126,
                  255,
                  0,
                  169
                ]
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
          "name": "stakeVault",
          "writable": true
        },
        {
          "name": "rewardVault",
          "writable": true
        },
        {
          "name": "tokenProgram",
          "address": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
        }
      ],
      "args": [
        {
          "name": "amount",
          "type": "u64"
        }
      ]
    }
  ],
  "accounts": [
    {
      "name": "pool",
      "discriminator": [
        241,
        154,
        109,
        4,
        17,
        177,
        109,
        188
      ]
    },
    {
      "name": "position",
      "discriminator": [
        170,
        188,
        143,
        228,
        122,
        64,
        247,
        208
      ]
    },
    {
      "name": "proposal",
      "discriminator": [
        26,
        94,
        189,
        187,
        116,
        136,
        53,
        33
      ]
    }
  ],
  "events": [
    {
      "name": "adminReplaced",
      "discriminator": [
        7,
        232,
        168,
        140,
        15,
        180,
        107,
        25
      ]
    },
    {
      "name": "emergencyWithdrawn",
      "discriminator": [
        116,
        226,
        36,
        3,
        37,
        92,
        138,
        76
      ]
    },
    {
      "name": "poolInitialized",
      "discriminator": [
        100,
        118,
        173,
        87,
        12,
        198,
        254,
        229
      ]
    },
    {
      "name": "poolPaused",
      "discriminator": [
        228,
        218,
        62,
        53,
        29,
        211,
        159,
        236
      ]
    },
    {
      "name": "poolUnpaused",
      "discriminator": [
        193,
        202,
        163,
        157,
        221,
        87,
        172,
        100
      ]
    },
    {
      "name": "positionClosed",
      "discriminator": [
        157,
        163,
        227,
        228,
        13,
        97,
        138,
        121
      ]
    },
    {
      "name": "positionOpened",
      "discriminator": [
        237,
        175,
        243,
        230,
        147,
        117,
        101,
        121
      ]
    },
    {
      "name": "proposalApproved",
      "discriminator": [
        70,
        49,
        155,
        228,
        157,
        43,
        88,
        49
      ]
    },
    {
      "name": "proposalClosed",
      "discriminator": [
        57,
        90,
        47,
        164,
        105,
        55,
        97,
        27
      ]
    },
    {
      "name": "proposalCreated",
      "discriminator": [
        186,
        8,
        160,
        108,
        81,
        13,
        51,
        206
      ]
    },
    {
      "name": "proposalExecuted",
      "discriminator": [
        92,
        213,
        189,
        201,
        101,
        83,
        111,
        83
      ]
    },
    {
      "name": "rewardRateChanged",
      "discriminator": [
        205,
        131,
        65,
        52,
        199,
        73,
        225,
        50
      ]
    },
    {
      "name": "rewardsClaimed",
      "discriminator": [
        75,
        98,
        88,
        18,
        219,
        112,
        88,
        121
      ]
    },
    {
      "name": "rewardsFunded",
      "discriminator": [
        84,
        233,
        245,
        203,
        228,
        147,
        165,
        92
      ]
    },
    {
      "name": "staked",
      "discriminator": [
        11,
        146,
        45,
        205,
        230,
        58,
        213,
        240
      ]
    },
    {
      "name": "unstaked",
      "discriminator": [
        27,
        179,
        156,
        215,
        47,
        71,
        195,
        7
      ]
    }
  ],
  "errors": [
    {
      "code": 6000,
      "name": "arithmeticOverflow",
      "msg": "Arithmetic overflow"
    },
    {
      "code": 6001,
      "name": "arithmeticUnderflow",
      "msg": "Arithmetic underflow"
    },
    {
      "code": 6002,
      "name": "invalidAmount",
      "msg": "Amount must be greater than zero"
    },
    {
      "code": 6003,
      "name": "invalidTokenDecimals",
      "msg": "Token mint must use exactly six decimals"
    },
    {
      "code": 6004,
      "name": "invalidAdminSet",
      "msg": "Admin set must contain exactly three distinct non-default keys"
    },
    {
      "code": 6005,
      "name": "invalidMintPair",
      "msg": "Stake and reward mints must be distinct"
    },
    {
      "code": 6006,
      "name": "invalidTokenProgram",
      "msg": "Only the original SPL Token Program is supported"
    },
    {
      "code": 6007,
      "name": "rewardRateAboveMaximum",
      "msg": "Reward rate is above the configured Devnet maximum"
    },
    {
      "code": 6008,
      "name": "nothingToClaim",
      "msg": "No whole reward base units are claimable"
    },
    {
      "code": 6009,
      "name": "unauthorized",
      "msg": "Signer is not authorized for this account"
    },
    {
      "code": 6010,
      "name": "positionNotEmpty",
      "msg": "Position still has stake or reward accounting"
    },
    {
      "code": 6011,
      "name": "poolPaused",
      "msg": "Pool is paused"
    },
    {
      "code": 6012,
      "name": "poolNotPaused",
      "msg": "Pool is not paused"
    },
    {
      "code": 6013,
      "name": "insufficientStake",
      "msg": "Position does not have enough staked principal"
    },
    {
      "code": 6014,
      "name": "insufficientRewardBacking",
      "msg": "Reward vault cannot back the requested payout"
    },
    {
      "code": 6015,
      "name": "proposalExpired",
      "msg": "Proposal has expired"
    },
    {
      "code": 6016,
      "name": "proposalAlreadyExecuted",
      "msg": "Proposal has already executed"
    },
    {
      "code": 6017,
      "name": "proposalNotApproved",
      "msg": "Proposal does not have enough approvals"
    },
    {
      "code": 6018,
      "name": "duplicateApproval",
      "msg": "Admin already approved this proposal"
    },
    {
      "code": 6019,
      "name": "staleProposal",
      "msg": "Proposal belongs to a stale admin epoch"
    },
    {
      "code": 6020,
      "name": "invalidProposalId",
      "msg": "Proposal id does not match the pool sequence"
    },
    {
      "code": 6021,
      "name": "invalidAdminReplacement",
      "msg": "Admin replacement is invalid"
    }
  ],
  "types": [
    {
      "name": "adminReplaced",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "proposal",
            "type": "pubkey"
          },
          {
            "name": "proposalId",
            "type": "u64"
          },
          {
            "name": "oldAdmin",
            "type": "pubkey"
          },
          {
            "name": "newAdmin",
            "type": "pubkey"
          },
          {
            "name": "adminEpoch",
            "type": "u64"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "emergencyWithdrawn",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "forfeitedScaled",
            "type": "u128"
          },
          {
            "name": "totalStaked",
            "type": "u64"
          },
          {
            "name": "remainingRewardBudgetScaled",
            "type": "u128"
          },
          {
            "name": "allocatedLiabilityScaled",
            "type": "u128"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "pool",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "type": "u8"
          },
          {
            "name": "initializer",
            "type": "pubkey"
          },
          {
            "name": "poolId",
            "type": "u64"
          },
          {
            "name": "poolBump",
            "type": "u8"
          },
          {
            "name": "poolAuthorityBump",
            "type": "u8"
          },
          {
            "name": "stakeMint",
            "type": "pubkey"
          },
          {
            "name": "rewardMint",
            "type": "pubkey"
          },
          {
            "name": "stakeVault",
            "type": "pubkey"
          },
          {
            "name": "rewardVault",
            "type": "pubkey"
          },
          {
            "name": "admins",
            "type": {
              "array": [
                "pubkey",
                3
              ]
            }
          },
          {
            "name": "adminEpoch",
            "type": "u64"
          },
          {
            "name": "nextProposalId",
            "type": "u64"
          },
          {
            "name": "paused",
            "type": "bool"
          },
          {
            "name": "maxRewardRatePerSlot",
            "type": "u64"
          },
          {
            "name": "rewardRatePerSlot",
            "type": "u64"
          },
          {
            "name": "lastUpdateSlot",
            "type": "u64"
          },
          {
            "name": "totalStaked",
            "type": "u64"
          },
          {
            "name": "accRewardPerStakeScaled",
            "type": "u128"
          },
          {
            "name": "remainingRewardBudgetScaled",
            "type": "u128"
          },
          {
            "name": "allocatedLiabilityScaled",
            "type": "u128"
          }
        ]
      }
    },
    {
      "name": "poolInitialized",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "initializer",
            "type": "pubkey"
          },
          {
            "name": "poolId",
            "type": "u64"
          },
          {
            "name": "stakeMint",
            "type": "pubkey"
          },
          {
            "name": "rewardMint",
            "type": "pubkey"
          },
          {
            "name": "stakeVault",
            "type": "pubkey"
          },
          {
            "name": "rewardVault",
            "type": "pubkey"
          },
          {
            "name": "maxRewardRatePerSlot",
            "type": "u64"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "poolPaused",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "poolUnpaused",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "proposal",
            "type": "pubkey"
          },
          {
            "name": "proposalId",
            "type": "u64"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "position",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "type": "u8"
          },
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "bump",
            "type": "u8"
          },
          {
            "name": "stakedAmount",
            "type": "u64"
          },
          {
            "name": "rewardDebtScaled",
            "type": "u128"
          },
          {
            "name": "pendingRewardScaled",
            "type": "u128"
          }
        ]
      }
    },
    {
      "name": "positionClosed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "positionOpened",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "proposal",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "version",
            "type": "u8"
          },
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "proposalId",
            "type": "u64"
          },
          {
            "name": "creator",
            "type": "pubkey"
          },
          {
            "name": "adminEpoch",
            "type": "u64"
          },
          {
            "name": "action",
            "type": {
              "defined": {
                "name": "proposalAction"
              }
            }
          },
          {
            "name": "approvals",
            "type": {
              "array": [
                "bool",
                3
              ]
            }
          },
          {
            "name": "approvalCount",
            "type": "u8"
          },
          {
            "name": "createdSlot",
            "type": "u64"
          },
          {
            "name": "expiresAtSlot",
            "type": "u64"
          },
          {
            "name": "executed",
            "type": "bool"
          },
          {
            "name": "bump",
            "type": "u8"
          }
        ]
      }
    },
    {
      "name": "proposalAction",
      "type": {
        "kind": "enum",
        "variants": [
          {
            "name": "setRewardRate",
            "fields": [
              {
                "name": "newRate",
                "type": "u64"
              }
            ]
          },
          {
            "name": "unpausePool"
          },
          {
            "name": "replaceAdmin",
            "fields": [
              {
                "name": "oldAdmin",
                "type": "pubkey"
              },
              {
                "name": "newAdmin",
                "type": "pubkey"
              }
            ]
          }
        ]
      }
    },
    {
      "name": "proposalApproved",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "proposal",
            "type": "pubkey"
          },
          {
            "name": "proposalId",
            "type": "u64"
          },
          {
            "name": "admin",
            "type": "pubkey"
          },
          {
            "name": "approvalCount",
            "type": "u8"
          },
          {
            "name": "adminEpoch",
            "type": "u64"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "proposalClosed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "proposal",
            "type": "pubkey"
          },
          {
            "name": "proposalId",
            "type": "u64"
          },
          {
            "name": "creator",
            "type": "pubkey"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "proposalCreated",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "proposal",
            "type": "pubkey"
          },
          {
            "name": "proposalId",
            "type": "u64"
          },
          {
            "name": "creator",
            "type": "pubkey"
          },
          {
            "name": "adminEpoch",
            "type": "u64"
          },
          {
            "name": "expiresAtSlot",
            "type": "u64"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "proposalExecuted",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "proposal",
            "type": "pubkey"
          },
          {
            "name": "proposalId",
            "type": "u64"
          },
          {
            "name": "adminEpoch",
            "type": "u64"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "rewardRateChanged",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "proposal",
            "type": "pubkey"
          },
          {
            "name": "proposalId",
            "type": "u64"
          },
          {
            "name": "newRate",
            "type": "u64"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "rewardsClaimed",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "paidScaled",
            "type": "u128"
          },
          {
            "name": "pendingRewardScaled",
            "type": "u128"
          },
          {
            "name": "allocatedLiabilityScaled",
            "type": "u128"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "rewardsFunded",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "funder",
            "type": "pubkey"
          },
          {
            "name": "sourceRewardAccount",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "remainingRewardBudgetScaled",
            "type": "u128"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "staked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "positionStakedAmount",
            "type": "u64"
          },
          {
            "name": "totalStaked",
            "type": "u64"
          },
          {
            "name": "slot",
            "type": "u64"
          }
        ]
      }
    },
    {
      "name": "unstaked",
      "type": {
        "kind": "struct",
        "fields": [
          {
            "name": "pool",
            "type": "pubkey"
          },
          {
            "name": "position",
            "type": "pubkey"
          },
          {
            "name": "owner",
            "type": "pubkey"
          },
          {
            "name": "amount",
            "type": "u64"
          },
          {
            "name": "positionStakedAmount",
            "type": "u64"
          },
          {
            "name": "totalStaked",
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

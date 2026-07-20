# CrossPoint Devnet demo transactions

Program id: `B2V3qSDVknbsfLL1ZGfcRxjQqnvUNRppF76X1YXEPNLF`

Produced by running `cargo run --manifest-path cli/Cargo.toml -- --cluster devnet --keypair ~/.config/solana/id.json`, which drives the full flow end to end: two merchants register, a customer enrols at both, earns 150 points at merchant A, the merchants open a bidirectional trade lane, the customer swaps 50 of those points into merchant B's points, claims an achievement badge, then redeems 20 remaining points.

| Step | Signature | Explorer link |
|---|---|---|
| register_merchant (A) | `4biBL3FdyXAprAQxbyyxFQzNE9vtbShT7Wi7SxL73rvBGLifDPkLmYiwwje3Q8Peep2bthy4BACtvwnh934nPNa2` | https://explorer.solana.com/tx/4biBL3FdyXAprAQxbyyxFQzNE9vtbShT7Wi7SxL73rvBGLifDPkLmYiwwje3Q8Peep2bthy4BACtvwnh934nPNa2?cluster=devnet |
| register_merchant (B) | `47FRrTP4YBxeApJ83mVibHPYHv9epkzp8oKB2eDHXC76uFiY17Zn9tCrWKzKCmQGXqSsHDjVZ51BEZDd4RM6Qvap` | https://explorer.solana.com/tx/47FRrTP4YBxeApJ83mVibHPYHv9epkzp8oKB2eDHXC76uFiY17Zn9tCrWKzKCmQGXqSsHDjVZ51BEZDd4RM6Qvap?cluster=devnet |
| enroll_customer (A) | `3D8P7RQgcd7aXuXf46qWm2qzVQ1Reqwg9CoRyhjkUWS7wfLrGQJcE2oZuyn6LJWuD2zj2xaKrhmFnLjP1SQdqFp5` | https://explorer.solana.com/tx/3D8P7RQgcd7aXuXf46qWm2qzVQ1Reqwg9CoRyhjkUWS7wfLrGQJcE2oZuyn6LJWuD2zj2xaKrhmFnLjP1SQdqFp5?cluster=devnet |
| enroll_customer (B) | `4VMR1YbxKcHEvaLDYTiyvndYvuG1d3B9ZB78nvJz1vb6oereiQMdLAh74ggaxFNe4dvpAPvgHaB9rs6JRs8nUPJm` | https://explorer.solana.com/tx/4VMR1YbxKcHEvaLDYTiyvndYvuG1d3B9ZB78nvJz1vb6oereiQMdLAh74ggaxFNe4dvpAPvgHaB9rs6JRs8nUPJm?cluster=devnet |
| record_purchase | `5B7jw75QfvvaNPFu1e3hSRVN5VhZa5Q6uGsBKEaUx3sDPkLb2ntm8KLv17wrxDXiUUSMcb5tYVq2F9h24fubeHVB` | https://explorer.solana.com/tx/5B7jw75QfvvaNPFu1e3hSRVN5VhZa5Q6uGsBKEaUx3sDPkLb2ntm8KLv17wrxDXiUUSMcb5tYVq2F9h24fubeHVB?cluster=devnet |
| propose_lane | `26W6HeqmoPPw6K5W1E5ZNyQNXc1bLHAK8wgMhdsx5Qa8CJMFj2fkrb5L5mSc9ABoiQCnDApPk41koGpBvK3bSbTH` | https://explorer.solana.com/tx/26W6HeqmoPPw6K5W1E5ZNyQNXc1bLHAK8wgMhdsx5Qa8CJMFj2fkrb5L5mSc9ABoiQCnDApPk41koGpBvK3bSbTH?cluster=devnet |
| accept_lane | `3uqc8WGbhJEmh43qPcSyFniyuF59ZR3rV2xk3H3cv3ss6PMHygNX4nVz9ehasSQdPdswUf3dkazZyPyJWyg1B6q5` | https://explorer.solana.com/tx/3uqc8WGbhJEmh43qPcSyFniyuF59ZR3rV2xk3H3cv3ss6PMHygNX4nVz9ehasSQdPdswUf3dkazZyPyJWyg1B6q5?cluster=devnet |
| swap_points | `4ZG58rGyc9Nxjj3UdrZgTX2iGFa9MNb5LddgRQExvbNox9Y9k5HGJ28K6ETw3MBFSyjXVg5SqWeb8izz6Q4cDB2r` | https://explorer.solana.com/tx/4ZG58rGyc9Nxjj3UdrZgTX2iGFa9MNb5LddgRQExvbNox9Y9k5HGJ28K6ETw3MBFSyjXVg5SqWeb8izz6Q4cDB2r?cluster=devnet |
| claim_achievement | `CtU4f1LvaXEhit5uyJwTUbYJyprns3KrAkPYczNQ1iSkMwJorrB2JLEZdfuTHYcdYCwkKCvYTzYhHKSL48k1bJW` | https://explorer.solana.com/tx/CtU4f1LvaXEhit5uyJwTUbYJyprns3KrAkPYczNQ1iSkMwJorrB2JLEZdfuTHYcdYCwkKCvYTzYhHKSL48k1bJW?cluster=devnet |
| redeem_points | `fy1exGFe5KbL9U3rCsJYW19jjw9tvgtxk12VivnEqiMMPFA8fL7FQp5m7HLfwALBtEBFjoHqcCuZGE3veHrobhu` | https://explorer.solana.com/tx/fy1exGFe5KbL9U3rCsJYW19jjw9tvgtxk12VivnEqiMMPFA8fL7FQp5m7HLfwALBtEBFjoHqcCuZGE3veHrobhu?cluster=devnet |

Final on-chain balances after the full run: 0.00008 merchant A points (150 earned, 50 swapped out, 20 redeemed), 0.000025 merchant B points (50 swapped in at a 0.5x rate). Both figures are UI amounts scaled by the fixed 6 decimals every points mint uses.

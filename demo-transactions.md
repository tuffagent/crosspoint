# CrossPoint Devnet demo transactions

Program id: `B2V3qSDVknbsfLL1ZGfcRxjQqnvUNRppF76X1YXEPNLF`

Produced by running `cargo run --manifest-path cli/Cargo.toml -- --cluster devnet --keypair ~/.config/solana/id.json`, which drives the full flow end to end: two merchants register, a customer enrols at both, earns 150 points at merchant A, the merchants open a bidirectional trade lane, the customer swaps 50 of those points into merchant B's points, claims an achievement badge, then redeems 20 remaining points.

| Step | Signature | Explorer link |
|---|---|---|
| register_merchant (A) | `3npeGCVsvn98sC7SyerFvLSou92dDNMgewSriL91MMsWz8GZT1fZokiCZjf1ZDdaPF8gFALcjsCU9kfkJkK8jLDh` | https://explorer.solana.com/tx/3npeGCVsvn98sC7SyerFvLSou92dDNMgewSriL91MMsWz8GZT1fZokiCZjf1ZDdaPF8gFALcjsCU9kfkJkK8jLDh?cluster=devnet |
| register_merchant (B) | `3uZRN5GkXoag5T1ZXFbPn54wgpsPgd4Er99PevakpZ3EFuTVHbo64W11SSwF55dNysXM8fg48fB9Hn9LNrf2f1RR` | https://explorer.solana.com/tx/3uZRN5GkXoag5T1ZXFbPn54wgpsPgd4Er99PevakpZ3EFuTVHbo64W11SSwF55dNysXM8fg48fB9Hn9LNrf2f1RR?cluster=devnet |
| enroll_customer (A) | `LnAd7rj9sKiqxpzLKfuMYMjQPHvzws5kx2udd8bsjyPPKXrpXERa4ok5cWPP9e52Y5vuLNcCrfTvGjDcNWzpYXH` | https://explorer.solana.com/tx/LnAd7rj9sKiqxpzLKfuMYMjQPHvzws5kx2udd8bsjyPPKXrpXERa4ok5cWPP9e52Y5vuLNcCrfTvGjDcNWzpYXH?cluster=devnet |
| enroll_customer (B) | `4h1T2PKXXkPkYJbW6wZ6PxzBovFAT2G769HKZTyTGkwB33cFucN8s1qRqhvetZZ6VM94hw4XSDF72MxT1ygwYuU1` | https://explorer.solana.com/tx/4h1T2PKXXkPkYJbW6wZ6PxzBovFAT2G769HKZTyTGkwB33cFucN8s1qRqhvetZZ6VM94hw4XSDF72MxT1ygwYuU1?cluster=devnet |
| record_purchase | `5mDfsEhjvEeJhEcCb1oSnFJcoG9wSfSFPRmSdGKuvwkVazdgSw5M7ox3q15Gud8gaweoVC8cEnXRwUPWTnuEvXgf` | https://explorer.solana.com/tx/5mDfsEhjvEeJhEcCb1oSnFJcoG9wSfSFPRmSdGKuvwkVazdgSw5M7ox3q15Gud8gaweoVC8cEnXRwUPWTnuEvXgf?cluster=devnet |
| propose_lane | `5egJoyyAxBG9pqTaymgkjALx8EbWJBfvrdmUgtBMe4hFAYAGugrQpcoUtxN5e2nCJuouc28EfcvUof1DjYmKsa1p` | https://explorer.solana.com/tx/5egJoyyAxBG9pqTaymgkjALx8EbWJBfvrdmUgtBMe4hFAYAGugrQpcoUtxN5e2nCJuouc28EfcvUof1DjYmKsa1p?cluster=devnet |
| accept_lane | `3NvEMKoW4MR46xRn4DQ3DpuaXVdcquLS9Tom7aveUJ9puqxcajFvxo9bGVAWu9xz3MEapSHJNTE7rqwPiEvgUso7` | https://explorer.solana.com/tx/3NvEMKoW4MR46xRn4DQ3DpuaXVdcquLS9Tom7aveUJ9puqxcajFvxo9bGVAWu9xz3MEapSHJNTE7rqwPiEvgUso7?cluster=devnet |
| swap_points | `2HJf1SM8CVLwu1UELip3sdjGxuWdwHKrqnFowq4sKyCg47X3TrZ6eQPYGsFBJ4ZWJCurGnh1f5daFS3yhWtafhb` | https://explorer.solana.com/tx/2HJf1SM8CVLwu1UELip3sdjGxuWdwHKrqnFowq4sKyCg47X3TrZ6eQPYGsFBJ4ZWJCurGnh1f5daFS3yhWtafhb?cluster=devnet |
| claim_achievement | `4w6jUFZxa1k1Gp4Gb8UB782GTVGj1PPnUjSft1fU4jf8QRLUUhNLgwBdfYczreALZLda47sF3Pyy7r56qdLq2ZGZ` | https://explorer.solana.com/tx/4w6jUFZxa1k1Gp4Gb8UB782GTVGj1PPnUjSft1fU4jf8QRLUUhNLgwBdfYczreALZLda47sF3Pyy7r56qdLq2ZGZ?cluster=devnet |
| redeem_points | `Q76x1GRB6kr3kvFMVqdQQCBpbgMKqx2FmB8p4P6XewJpW5NQvYWnEk6Ex7ALpLf4NqmcTFGXS11P6dDsx9o854F` | https://explorer.solana.com/tx/Q76x1GRB6kr3kvFMVqdQQCBpbgMKqx2FmB8p4P6XewJpW5NQvYWnEk6Ex7ALpLf4NqmcTFGXS11P6dDsx9o854F?cluster=devnet |

Final on-chain balances after the full run: 0.00008 merchant A points (150 earned, 50 swapped out, 20 redeemed), 0.000025 merchant B points (50 swapped in at a 0.5x rate). Both figures are UI amounts scaled by the fixed 6 decimals every points mint uses.

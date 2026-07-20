# CrossPoint

CrossPoint is a cross-merchant loyalty-points programme for Solana, built with Anchor and Token-2022. Instead of each merchant running an isolated, siloed points ledger, CrossPoint lets independent merchants opt into bidirectional trade lanes so a customer can swap points earned at one shop into points spendable at another, at a rate the two merchants agree on together. Customers who cross lifetime spending or trading thresholds can claim soulbound achievement badges, enforced on-chain by the token programme itself rather than by convention.

## The novel mechanism

Two ideas sit at the centre of CrossPoint:

- **Cross-merchant swap lanes.** Any two registered merchants can open a `TradeLane`: merchant A proposes a pair of exchange rates (one for each direction), merchant B accepts, and only then can customers swap between the two merchants' points at that rate. Neither merchant can set a one-sided rate unilaterally, since activation needs both signatures. Rates aren't edited in place; a lane is closed and reopened via a fresh propose/accept cycle, so a rate change is always a deliberate, re-agreed act rather than a silent update.
- **Customer-claimed soulbound achievements.** Rather than a merchant or the programme auto-minting a badge the moment a threshold is crossed (which would force whoever triggers the mint to pay for an account the customer might not even want), the customer claims their own badge with a dedicated instruction, once eligible. The badge itself cannot be transferred, gifted or sold afterwards.

## How Solana's infrastructure specifically enables this

- **Token-2022's MetadataPointer and on-chain TokenMetadata extensions** let every points mint and every badge mint carry its own name and symbol directly in the mint account, with no off-chain metadata service or IPFS pin required for a wallet or explorer to display it sensibly.
- **The NonTransferable extension** is what actually makes an achievement badge soulbound. This is not a convention enforced by CrossPoint's own instructions; it is enforced by the SPL Token-2022 programme itself at the CPI level, so no other programme (CrossPoint included) can move a claimed badge out of the customer's wallet, even if it tried to.
- **PDA-signed cross-programme invocation** is what lets a single `swap_points` call burn one merchant's points and mint another merchant's points atomically, with the programme itself as mint authority on both sides, no merchant needing to be online or sign anything at swap time.

## Trade-offs (documented deliberately, not discovered by accident)

**Rent payer strategy.** Every new account CrossPoint creates on a customer's behalf, their points token account, their `CustomerStats` record, an achievement badge mint and its token account, is paid for by the customer, never the merchant. A customer's very first purchase at a given merchant therefore needs the customer's own signature and funds (`enroll_customer`), after which the merchant alone can record further purchases with no new rent involved. The alternative, a merchant always paying rent for new customer accounts, opens a Sybil hole: an attacker can spin up unlimited throwaway wallets and drain a merchant's SOL one dust purchase at a time. Requiring whoever benefits from a new account to fund it closes that hole, at the cost of one extra signature the first time a customer meets a new merchant, which is already a familiar pattern on Solana (receiving any new SPL token typically requires the recipient to fund their own token account).

**Fixed token decimals.** Every points and badge mint CrossPoint creates uses a hardcoded 6 decimals; merchants cannot choose their own granularity. This means the swap rate maths is always a straightforward `amount * rate / 1_000_000` on both sides of a lane, with no need to read either mint's decimals at swap time or normalise between mismatched values. The cost is that merchants lose a small degree of customisation; the benefit is removing an entire class of decimal-mismatch bugs from the code that handles real value transfer.

## Instructions

| Instruction | Who calls it | Effect |
|---|---|---|
| `register_merchant(name, symbol, uri)` | Merchant authority | Creates the `Merchant` PDA and a Token-2022 points mint (fixed 6 decimals, on-chain metadata). One registration per authority. |
| `enroll_customer()` | Customer | Funds and creates the customer's `CustomerStats` record and points token account at a given merchant, the customer's one-off setup cost the first time they interact with a new merchant. |
| `record_purchase(amount)` | Merchant authority | Mints `amount` points to an already-enrolled customer's token account and updates their lifetime-earned total. Callable by an external point-of-sale system via CPI, since it needs only the merchant's signature once a customer has enrolled. |
| `propose_lane(rate_a_to_b, rate_b_to_a)` | The lower-pubkey merchant's authority | Creates an inactive `TradeLane` between two merchants with the proposed pair of exchange rates, or reopens negotiation if the lane already exists and is currently inactive. |
| `accept_lane()` | The higher-pubkey merchant's authority | Activates a proposed lane, making swaps between the two merchants possible. |
| `close_lane()` | Either merchant's authority | Deactivates a lane without deleting it, so it can later be re-proposed with new rates. |
| `swap_points(amount)` | Customer | Burns `amount` points at one merchant and mints the rate-converted amount at the other, in whichever direction the customer chooses, over an active lane. |
| `redeem_points(amount)` | Customer | Burns points at the originating merchant to represent a redeemed reward. |
| `claim_achievement(badge_id)` | Customer | Checks the relevant threshold against the customer's stats, mints a soulbound (NonTransferable) badge to the customer's own token account, and rejects a repeat claim of the same badge. |

Three achievement badges are defined: Frequent Customer (100 lifetime points earned at a single merchant), Loyal Patron (500 lifetime points earned at a single merchant), and Cross-Merchant Trader (at least one successful swap).

## Deployed programme

- **Devnet programme id:** `B2V3qSDVknbsfLL1ZGfcRxjQqnvUNRppF76X1YXEPNLF`
- **Demo transactions:** see [`demo-transactions.md`](./demo-transactions.md) for every instruction's signature and Solana Explorer link from a real end-to-end run of the CLI against Devnet.

## Running the demo client

The `cli` crate drives the entire flow (register two merchants, enrol a customer at both, record a purchase, open and accept a trade lane, swap, claim a badge, redeem) against whichever cluster you point it at, printing every transaction signature and the resulting balances.

```bash
cargo run --manifest-path cli/Cargo.toml -- --cluster devnet --keypair ~/.config/solana/id.json
```

`--cluster` accepts any `anchor-client` cluster moniker (`devnet`, `localnet`, `mainnet`, or a custom URL) and defaults to `devnet`. `--keypair` points at a funded Solana wallet used to pay for the demo's throwaway accounts; if omitted, a fresh, unfunded keypair is generated, which will fail on the first transaction.

## Running the programme's own tests

```bash
cargo build-sbf --manifest-path programs/crosspoint/Cargo.toml
cargo test --manifest-path programs/crosspoint/Cargo.toml
```

Tests run against `litesvm`, a Rust-native, in-process Solana runtime, so no local validator or network access is required.

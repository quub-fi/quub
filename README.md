# Quub

Quub is a payments fabric. There is no native token.

Quub is the payments fabric: ledger (Rust node + EVM + Solidity facades) plus Quub Policy, Quub Memo / Quub ISO, Quub Evidence, and Quub Gateway. **fazeZERO** is the company and is not in this repository.

## Sprint 1.5

- Reth pin: `paradigmxyz/reth` git tag **`v2.5.2`**.
- Rust toolchain: **1.95.0**.
- `quub-node` launches an `EthereumNode` with `QuubExecutorBuilder` (no `op-node`, no Simplex).
- HTTP JSON-RPC: `http://127.0.0.1:8545`, chain id **8091**.
- Precompiles F201–F203 registered for `spec >= Prague` (`new_stateful` for F201/F202).
- System contracts etched at genesis: **F210–F213** (see `crates/quub-node/alloc/`).

### --dev key (public anvil account 0 — not production)

- Address: `0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266`
- Private key: `0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80`
- PolicyAdmin OwnerA = OwnerB = this key is a **`--dev` bypass**. Production dual-control + 24h timelock is Sprint 2+.

### Frozen addresses

| Addr | Role |
|------|------|
| `0x…F201` | Quub Policy precompile |
| `0x…F202` | Quub ISO memo precompile |
| `0x…F203` | Quub Paymaster precompile |
| `0x…F210` | PaymentToken |
| `0x…F211` | PolicyAdmin |
| `0x…F212` | EvidenceAnchor |
| `0x…F213` | PaymasterEntry |

### Memo hash origins

`memoHash = keccak256(abi.encode(endToEndId, uetr, instrId, ccy, msgType, tx.origin))`

- Fixture `usd_pacs008_stable_hash` uses origin `0x…00AA`.
- A mined `transferWithMemo` from the anvil key uses origin `0xf39F…` — **not** equal to the fixture.
- `cast call` F202 `--from 0xf39F…` must match the receipt `MemoAnchored` hash.

## Build / test

```bash
cargo test --workspace
cd contracts && forge test
```

## Local payment (Sprint 1.5)

```bash
bash scripts/devnet.sh
```

Or manually:

```bash
# terminal 1
cargo run -p quub-node

# terminal 2
cast chain-id --rpc-url http://127.0.0.1:8545   # expect 8091
cast code 0x000000000000000000000000000000000000F210 --rpc-url http://127.0.0.1:8545
cast send 0x000000000000000000000000000000000000F210 \
  "transferWithMemo(address,uint256,bytes32,bytes16,bytes32,bytes3,uint8,bytes32,bytes32)" \
  0x70997970C51812dc3A010C7d01b50e0d17dc79C8 \
  1000000 \
  0x1111111111111111111111111111111111111111111111111111111111111111 \
  0x22222222222222222222222222222222 \
  0x3333333333333333333333333333333333333333333333333333333333333333 \
  0x555344 \
  0 \
  0x0000000000000000000000000000000000000000000000000000000000000000 \
  0x4444444444444444444444444444444444444444444444444444444444444444 \
  --rpc-url http://127.0.0.1:8545 \
  --private-key 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
```

Send to **F210**, not F201. Raw `cast call` F201 from the anvil key empty-reverts (caller must be F210).

## Locked stack

Rust + Solidity + Foundry. Mode A (OP Stack) is the default **crate feature** (`mode-a` compiles a stub; launch does not call it). Mode B (Simplex) is feature-flagged and never enabled in the same binary. Not Substrate. Not a custom VM.

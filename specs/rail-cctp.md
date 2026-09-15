# Rail: CCTP V2 → Base Sepolia (Sprint 6 / ADR-020)

Quub **records** policy + memo + evidence on chain **8091**. Circle **CCTP V2** moves USDC.
Quub is not a bridge. Quub 8091 is **not** a CCTP domain.

## Cite

Primary pin source (Circle / circlefin CCTP V2 testnet tables, mirrored in `circlefin/cctp-go`):

- <https://developers.circle.com/cctp/evm-smart-contracts>
- <https://github.com/circlefin/cctp-go/blob/main/chains.go>

V1 messengers are different addresses. This rail uses **V2 only**.

## Domains and chain ids

| Role | Network | CCTP domain | Chain id |
|------|---------|-------------|----------|
| Identity (Quub) | Quub `--dev` | **n/a** | **8091** |
| CCTP burn (source) | Ethereum Sepolia | **0** | 11155111 |
| CCTP mint (dest) | Base Sepolia | **6** | 84532 |

## CCTP V2 contracts (all V2 testnets share these)

| Contract | Address |
|----------|---------|
| TokenMessengerV2 | `0x8FE6B999Dc680CcFDD5Bf7EB0974218be2542DAA` |
| MessageTransmitterV2 | `0xE737e5cEBEEBa77EFE34D4aa090756590b1CE275` |

## USDC

| Network | USDC |
|---------|------|
| Ethereum Sepolia | `0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238` |
| Base Sepolia | `0x036CbD53842c5426634e7929541eC2318f3dCF7e` |

## Mock mode

Same struct fields. When `SEPOLIA_RPC` / `BASE_SEPOLIA_RPC` / `CIRCLE_API` are unset, adapters return synthetic burn/mint ids that are **not** equal to the Quub tx hash. Placeholders for unused live fields may be `0x` + zeros; domain ids and contract addresses above remain the pin even in mock.

## Env

| Var | Role |
|-----|------|
| `SEPOLIA_RPC` | Ethereum Sepolia (live burn) |
| `BASE_SEPOLIA_RPC` | Base Sepolia (live mint) |
| `CIRCLE_API` | Circle attestation API |
| Burner key | Funded Sepolia (and Base if needed); ≤1 USDC |

Mock requires none of these.

## RailRecord (by `endToEndId`)

`quub_tx`, `memo_hash`, `burn_tx`, `mint_tx`, `status` ∈ `quub_only | burned | minted | failed`.

If burn_tx == quub_tx → not CCTP → fail.

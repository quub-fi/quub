# Quub — Physical architecture
**Companion to** `DEPLOYMENT.md` (how to start it) and `ARCHITECTURE.md` (what it is).  
**As of** 15 September 2026. Year-1 is a **single workstation**. Production is a target topology, not standing kit.

Physical here means: hosts, processes on those hosts, disks, ports, and trust zones. It is not a cloud invoice.

---

## 1. What exists in the room

One engineer machine (`NR-MBPro`, Darwin arm64) runs every Quub process.

```
┌──────────────────────────────────────────────────────────────────┐
│  HOST  laptop-dev                                                │
│  OS    macOS 25.x  arm64                                         │
│  rustc 1.96.0   Foundry cast/forge 1.8.x                         │
│                                                                  │
│  DISK  repo + Cargo target + artifacts/mode-a + geth datadir     │
│  NET   loopback only (127.0.0.1). No public bind.                │
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐  │
│  │ quub-node    │  │ quub-node    │  │ geth 1.16.5            │  │
│  │ --dev        │  │ --engine     │  │ L1 datadir             │  │
│  │ :8545        │  │ :9545 :9551  │  │ :8546                  │  │
│  └──────────────┘  └──────▲───────┘  └──────────▲─────────────┘  │
│                           │ JWT                 │ L1 RPC         │
│                    ┌──────┴───────┐             │                │
│                    │ op-node      │─────────────┘                │
│                    │ v1.19.7      │                              │
│                    └──────────────┘                              │
│  ┌──────────────┐  ┌──────────────┐                              │
│  │ operator-api │  │ quub-gateway │  (Sprint 6; mock default)    │
│  │ :8080        │  │ CLI / lib    │                              │
│  └──────────────┘  └──────────────┘                              │
└──────────────────────────────────────────────────────────────────┘
```

There is no second machine, no load balancer, no colo, no HSM. Anvil is **not** a host on Mode A.

---

## 2. Host roles (logical — today they share one box)

| Role | Process | Bind | Disk | Trust |
|---|---|---|---|---|
| Dev execution | `quub-node --dev` | `127.0.0.1:8545` | ephemeral node datadir | engineer |
| L2 execution | `quub-node --engine` | `127.0.0.1:9545` HTTP, `:9551` Engine | Mode A L2 datadir | engineer |
| L2 consensus | `op-node` v1.19.7 | loopback to L1 + Engine | `rollup.json`, logs | engineer |
| L1 execution | `geth` 1.16.5 | `127.0.0.1:8546` | `artifacts/mode-a/l1` | engineer |
| Product API | `operator-api` | `127.0.0.1:8080` | memory only | bearer token |
| Rail worker | `quub-gateway` / `rail-base.sh` | outbound HTTPS if live CCTP | `artifacts/rail-base/` | Circle + burner key |
| Build | `cargo` / `forge` | none | `target/`, `contracts/out/` | engineer |

Same `quub-node` binary. Two **instances** if both `--dev` and `--engine` run — two datadirs, never one.

---

## 3. Network zones

```
                    untrusted internet
                           │
                           │  (not open today)
                           ▼
                 ┌─────────────────┐
                 │  future edge    │   TLS terminate
                 │  api. / rpc.    │   quub.network
                 └────────┬────────┘
                          │
            ┌─────────────┴──────────────┐
            │  loopback trust zone       │
            │  127.0.0.1                 │
            │  operator-api :8080        │
            │  quub-node HTTP            │
            └─────────────┬──────────────┘
                          │
            ┌─────────────┴──────────────┐
            │  engine trust zone         │
            │  :9551 + JWT file mode 600 │
            │  op-node ↔ quub-node       │
            │  never publish this port   │
            └─────────────┬──────────────┘
                          │
            ┌─────────────┴──────────────┐
            │  L1 trust zone             │
            │  geth :8546                │
            │  local only year-1         │
            └────────────────────────────┘

  outbound (Sprint 6 live only):
    Circle attestation API
    Ethereum Sepolia RPC   (CCTP burn — not Quub 8091)
    Base Sepolia RPC       (CCTP mint)
```

Year-1 rule: **nothing listens on `0.0.0.0`.** Physical isolation is “loopback + JWT,” not a firewall appliance.

---

## 4. Storage

| Volume / path | Contents | Lose it and… |
|---|---|---|
| git repo | source, ADRs, alloc hex, scripts | rebuild from origin |
| `target/debug/quub-node` | binary (~250 MB) | `cargo build -p quub-node` |
| `--dev` datadir | chain 8091 history | receipts vanish (expected) |
| `artifacts/mode-a/` | jwt, rollup.json, L1/L2 genesis, geth datadir, local bins | Mode A must be redeployed |
| `crates/quub-node/alloc/` | F210–F213 runtime + storageLayout | genesis etch breaks |
| `artifacts/rail-base/` | mock/live rail JSON | rail audit trail gone; Quub receipts remain |
| `artifacts/mode-a/bin/` | geth, sometimes op-node | install again; versions pinned in DEPLOYMENT.md |

No shared NFS. No object store. Backups today = git + a copy of `artifacts/mode-a` if you care about that L2.

---

## 5. Binaries on the box

| Binary | Source | Pin |
|---|---|---|
| `quub-node` | this repo | `op-rs/reth@aef8d3ef` + `op-reth/v2.4.4` |
| `quub-operator-api` | `apps/operator-api` | workspace |
| `op-node` | optimism `op-node/v1.19.7` source build | v1.19.7 |
| `geth` | go-ethereum | 1.16.5 |
| `op-deployer` | optimism | 0.8.0-rc.2 (Mode A genesis) |
| `cast` / `forge` | Foundry | 1.8.1 |
| `rustc` / `cargo` | rustup | 1.96.0 |

`op-node` v1.19.7 has no GitHub release asset — it is compiled on this host. Treat `artifacts/mode-a/bin/` as part of the physical install, not “whatever Homebrew has.”

---

## 6. Target physical topology (not purchased)

When a partner exists, **split the laptop into four hosts**. Do not invent a fifth “chain” host.

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ API          │     │ Sequencer    │     │ L1 + batch   │
│ operator-api │     │ quub-node    │     │ geth/reth    │
│ + TLS proxy  │     │ --engine     │     │ op-batcher   │
│ Postgres     │     │ op-node seq  │     │ op-proposer  │
│ 2 vCPU / 4G  │     │ JWT private  │     │ Sepolia first│
└──────────────┘     └──────────────┘     └──────────────┘
        │                    │                    │
        └────────── eth_* ───┘                    │
                         Engine API only on private NIC
                                                  │
                         ┌──────────────┐         │
                         │ Verifier     │         │
                         │ quub-node    │─────────┘
                         │ + op-node    │  L1 RPC
                         │ (no sequencer key)
                         └──────────────┘
```

| Host | Must not hold |
|---|---|
| API | Engine JWT, OwnerA/OwnerB, sequencer key |
| Sequencer | Circle burner, operator Bearer DB |
| L1/batch | F211 owner keys |
| Verifier | sequencer key, operator key |

Rail worker (`quub-gateway`) sits with API or a fifth small job box. It holds the CCTP burner — never the F211 owners.

Hardware bar for year-2 sequencer: 8+ cores, 32 GB RAM, NVMe for the Reth datadir. Laptop RAM is not a capacity plan.

---

## 7. Trust and key placement

| Material | Lives today | Must live next |
|---|---|---|
| anvil0 / anvil1 keys | scripts / env | nowhere |
| `QUUB_OPERATOR_KEY` | env on laptop | API host KMS |
| `QUUB_OPERATOR_TOKEN` | env | API secrets manager |
| Engine JWT | `artifacts/mode-a/jwt.txt` | sequencer + op-node disk, mode 600 |
| OwnerA / OwnerB | anvil keys on laptop | two HSMs or two people, not one VM |
| CCTP burner | unset or env | rail host only |
| Circle API token | unset or env | rail host only |

Physical rule: **the machine that can freeze is not the machine that can mint USDC.** Today both are the laptop. That is the first split.

---

## 8. Failure and locality

| Event | Physical effect |
|---|---|
| Laptop sleep | all processes stop; `--dev` may drop the datadir |
| Kill `op-node` | L2 head freezes (required). `:9545` still answers old state |
| Kill geth | `op-node` cannot derive; L2 stops after unsafe window |
| Wipe `artifacts/mode-a` | Mode A gone; `--dev` unaffected |
| Wipe `--dev` datadir | 8545 receipts gone; Mode A unaffected |
| No Circle / Sepolia | rail mock on disk; Quub still settles F210 |

Two datadirs exist so a Mode A experiment cannot delete the `--dev` payment story. Do not symlink them.

---

## 9. What this architecture refuses

- Anvil as a physical L1 for `op-node`
- `0.0.0.0` on 8545 / 9545 / 8080 / 9551
- Simplex or a second consensus binary on the same host “for HA”
- A third Reth build (Paradigm tag + `op-rs` + local fork)
- Hosting USDC custody on the Quub sequencer
- Calling 8091 a CCTP domain

When those show up in a diagram, the diagram is wrong — not “future physical.”

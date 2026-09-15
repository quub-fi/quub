//! Genesis alloc for F210–F213 + funded `--dev` anvil key.
//!
//! Runtime bytecode from `forge inspect … deployedBytecode` (no immutables).
//! Storage slots from committed `alloc/*.storageLayout.json`.

use alloy_genesis::{Genesis, GenesisAccount};
use alloy_primitives::{address, keccak256, Address, Bytes, B256, U256};
use std::collections::BTreeMap;

/// Public anvil account 0 — `--dev` only. Not production.
pub const DEVNET_ANVIL: Address = address!("0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

const F210: Address = address!("0x000000000000000000000000000000000000F210");
const F211: Address = address!("0x000000000000000000000000000000000000F211");
const F212: Address = address!("0x000000000000000000000000000000000000F212");
const F213: Address = address!("0x000000000000000000000000000000000000F213");

/// 1_000_000e6 payment-token units minted to the --dev key.
const MINT_SUPPLY: u128 = 1_000_000_000_000;

fn hex_runtime(s: &str) -> Bytes {
    let s = s.trim().trim_start_matches("0x");
    Bytes::from(hex::decode(s).expect("runtime hex"))
}

fn map_slot(account: Address, base: u64) -> B256 {
    let mut buf = [0u8; 64];
    buf[12..32].copy_from_slice(account.as_slice());
    buf[56..64].copy_from_slice(&base.to_be_bytes());
    keccak256(buf)
}

fn word_addr(a: Address) -> B256 {
    let mut w = [0u8; 32];
    w[12..32].copy_from_slice(a.as_slice());
    B256::from(w)
}

fn word_u256(v: U256) -> B256 {
    B256::from(v.to_be_bytes::<32>())
}

/// Slot 1 packing: ownerB at offset 0 (low 160 bits), paused at bit 160.
fn owner_b_paused_word(owner_b: Address, paused: bool) -> B256 {
    let mut n = U256::from_be_bytes({
        let mut t = [0u8; 32];
        t[12..32].copy_from_slice(owner_b.as_slice());
        t
    });
    // Left-padded address as BE U256 places the address in the *low* 160 bits. Correct for packing.
    if paused {
        n |= U256::from(1u64) << 160;
    }
    word_u256(n)
}

pub fn quub_genesis() -> Genesis {
    let mut genesis = Genesis::default().with_gas_limit(30_000_000);

    genesis.alloc.insert(
        DEVNET_ANVIL,
        GenesisAccount::default().with_balance(U256::from(10u64).pow(U256::from(21u64))),
    );

    let mut f211_storage = BTreeMap::new();
    f211_storage.insert(word_u256(U256::from(0u64)), word_addr(DEVNET_ANVIL));
    f211_storage.insert(
        word_u256(U256::from(1u64)),
        owner_b_paused_word(DEVNET_ANVIL, false),
    );
    f211_storage.insert(word_u256(U256::from(2u64)), B256::ZERO);
    f211_storage.insert(word_u256(U256::from(3u64)), word_addr(F210));
    genesis.alloc.insert(
        F211,
        GenesisAccount::default()
            .with_balance(U256::ZERO)
            .with_code(Some(hex_runtime(include_str!(
                "../alloc/PolicyAdmin.runtime.hex"
            ))))
            .with_storage(Some(f211_storage)),
    );

    let mut f210_storage = BTreeMap::new();
    f210_storage.insert(
        word_u256(U256::from(0u64)),
        word_u256(U256::from(MINT_SUPPLY)),
    );
    f210_storage.insert(
        map_slot(DEVNET_ANVIL, 1),
        word_u256(U256::from(MINT_SUPPLY)),
    );
    genesis.alloc.insert(
        F210,
        GenesisAccount::default()
            .with_balance(U256::ZERO)
            .with_code(Some(hex_runtime(include_str!(
                "../alloc/PaymentToken.runtime.hex"
            ))))
            .with_storage(Some(f210_storage)),
    );

    genesis.alloc.insert(
        F212,
        GenesisAccount::default()
            .with_balance(U256::ZERO)
            .with_code(Some(hex_runtime(include_str!(
                "../alloc/EvidenceAnchor.runtime.hex"
            )))),
    );

    let mut f213_storage = BTreeMap::new();
    f213_storage.insert(word_u256(U256::from(0u64)), word_addr(DEVNET_ANVIL));
    f213_storage.insert(word_u256(U256::from(1u64)), word_addr(DEVNET_ANVIL));
    f213_storage.insert(map_slot(F210, 2), word_u256(U256::from(1u64)));
    genesis.alloc.insert(
        F213,
        GenesisAccount::default()
            .with_balance(U256::ZERO)
            .with_code(Some(hex_runtime(include_str!(
                "../alloc/PaymasterEntry.runtime.hex"
            ))))
            .with_storage(Some(f213_storage)),
    );

    genesis
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alloc_has_system_contracts() {
        let g = quub_genesis();
        assert!(g.alloc.get(&F210).unwrap().code.as_ref().unwrap().len() > 10);
        assert!(g.alloc.get(&F211).unwrap().code.as_ref().unwrap().len() > 10);
        assert!(g.alloc.get(&F212).unwrap().code.as_ref().unwrap().len() > 10);
        assert!(g.alloc.get(&F213).unwrap().code.as_ref().unwrap().len() > 10);
        assert!(g.alloc.get(&DEVNET_ANVIL).unwrap().balance > U256::ZERO);
    }
}

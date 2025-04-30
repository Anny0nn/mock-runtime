#![cfg_attr(not(feature = "std"), no_std)]

// Make the WASM binary available.
#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

extern crate alloc;

use alloc::{vec, vec::Vec};
use codec::{Decode, Encode};
use log::info;
const LOG_TARGET: &str = "mock-runtime";
use polkadot_sdk::{
    self, frame_system_rpc_runtime_api,
    polkadot_sdk_frame::prelude::{Hash as HashT, ValidTransaction},
    sp_api::{self, impl_runtime_apis},
    sp_block_builder,
    sp_core::{self, OpaqueMetadata},
    sp_genesis_builder::{self, PresetId},
    sp_inherents, sp_io, sp_offchain,
    sp_runtime::{
        self,
        generic::{self},
        traits::{BlakeTwo256, Block as BlockT, IdentifyAccount, Verify},
        transaction_validity::TransactionValidity,
        ApplyExtrinsicResult, ExtrinsicInclusionMode, MultiAddress, MultiSignature,
        OpaqueExtrinsic,
    },
    sp_session, sp_std,
    sp_storage::well_known_keys,
    sp_transaction_pool,
    sp_version::{self, Cow, RuntimeVersion},
};

const HEADER_KEY: &[u8] = b":header";
pub const EXTRINSICS_KEY: &[u8] = b"extrinsics";

#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone)]
pub struct Runtime;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// The amount type, should be signed version of Balance.
pub type Amount = i128;

/// Nonce of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An index to a block.
pub type BlockNumber = u32;

/// The address format for describing accounts.
pub type Address = MultiAddress<AccountId, ()>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, OpaqueExtrinsic>;

/// A Block signed with a Justification
pub type SignedBlock = generic::SignedBlock<Block>;

/// BlockId type as expected by this runtime.
pub type BlockId = generic::BlockId<Block>;

/// This runtime version.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: Cow::Borrowed("mock-runtime"),
    impl_name: Cow::Borrowed("mock-runtime"),
    authoring_version: 1,
    spec_version: 1,
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    system_version: 1,
};

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime.
///
/// They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the dao data structures.
pub mod opaque {
    use super::*;
    use sp_runtime::{generic, traits::BlakeTwo256};

    pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
    /// Opaque block header type.
    pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
    /// Opaque block type.
    pub type Block = generic::Block<Header, UncheckedExtrinsic>;
    /// Opaque block identifier type.
    pub type BlockId = generic::BlockId<Block>;
}

/// Provides getters for genesis configuration presets.
pub mod genesis_config_presets {
    use super::*;

    use alloc::{vec, vec::Vec};
    use serde_json::Value;

    /// Returns a development genesis config preset.
    pub fn development_config_genesis() -> Value {
        let genesis = serde_json::json!({});
        genesis
    }

    /// Get the set of the available genesis config presets.
    pub fn get_preset(id: &PresetId) -> Option<Vec<u8>> {
        let patch = match id.as_ref() {
            sp_genesis_builder::DEV_RUNTIME_PRESET => development_config_genesis(),
            _ => development_config_genesis(),
        };
        Some(
            serde_json::to_string(&patch)
                .expect("serialization to json is expected to work. qed.")
                .into_bytes(),
        )
    }

    /// List of supported presets.
    pub fn preset_names() -> Vec<PresetId> {
        vec![PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET)]
    }
}

impl Runtime {
    pub fn get_state<T: Decode>(key: &[u8]) -> Option<T> {
        let data = sp_io::storage::get(key).unwrap();
        T::decode(&mut &*data).ok()
    }

    fn update_header(initial_header: Header) -> Header {
        let state_root = {
            let raw = &sp_io::storage::root(Default::default())[..];
            sp_core::H256::decode(&mut &raw[..]).unwrap()
        };

        let extrinsics = sp_io::storage::get(EXTRINSICS_KEY)
            .and_then(|bytes| <Vec<Vec<u8>> as Decode>::decode(&mut &*bytes).ok())
            .unwrap_or_default();

        let expected_extrinsics_root =
            BlakeTwo256::ordered_trie_root(extrinsics, Default::default());

        let mut header = initial_header;
        header.extrinsics_root = expected_extrinsics_root;
        header.state_root = state_root;

        header
    }

    fn do_validate_transaction(
        _tx: <Block as BlockT>::Extrinsic,
        _block_hash: <Block as BlockT>::Hash,
    ) -> TransactionValidity {
        Ok(ValidTransaction::default())
    }

    fn do_dispatch_extrinsic(_extrinsic: <Block as BlockT>::Extrinsic) {
        info!(target: LOG_TARGET, "Dispatching extrinsic: {:?}", _extrinsic);
    }

    pub fn do_execute_block(block: Block) {
        info!(target: LOG_TARGET, "Executing block number: {:?}", block.header.number);
        for extrinsic in block.extrinsics {
            Self::do_dispatch_extrinsic(extrinsic);
        }
    }

    pub fn do_initialize_block(header: &<Block as BlockT>::Header) -> ExtrinsicInclusionMode {
        info!(target: LOG_TARGET, "Initializing block number: {:?}", header.number);
        sp_io::storage::set(HEADER_KEY, &header.encode());
        sp_io::storage::clear(EXTRINSICS_KEY);
        ExtrinsicInclusionMode::AllExtrinsics
    }

    pub fn do_finalize_block() -> <Block as BlockT>::Header {
        let header = Self::get_state::<<Block as BlockT>::Header>(HEADER_KEY)
            .expect("Header should be initialized");

        sp_io::storage::clear(HEADER_KEY);

        Self::update_header(header)
    }
}

impl_runtime_apis! {
impl sp_api::Core<Block> for Runtime {
    fn version() -> RuntimeVersion {
        VERSION
    }

    fn execute_block(block: Block) {
        info!(
            target: LOG_TARGET,
            "Entering execute_block block: {:?} (exts: {})",
            block,
            block.extrinsics.len()
        );
        Self::do_execute_block(block)
    }

    fn initialize_block(header: &<Block as BlockT>::Header) -> ExtrinsicInclusionMode{
        info!(
            target: LOG_TARGET,
            "Entering initialize_block. header: {:?} / version: {:?}", header, VERSION.spec_version
        );
        Self::do_initialize_block(header)
    }
}

impl sp_block_builder::BlockBuilder<Block> for Runtime {
    fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
        info!(target: LOG_TARGET, "Entering apply_extrinsic: {:?}", extrinsic);
        Self::do_dispatch_extrinsic(extrinsic);
        Ok(Ok(()))
    }

    fn finalize_block() -> <Block as BlockT>::Header {
        let header = Self::do_finalize_block();
        info!(target: LOG_TARGET, "Finalized block authoring {:?}", header);
        header
    }

    fn inherent_extrinsics(_data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
        Default::default()
    }

    fn check_inherents(
        _block: Block,
        _data: sp_inherents::InherentData,
    ) -> sp_inherents::CheckInherentsResult {
        Default::default()
    }
}

impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
    fn validate_transaction(
        _source: sp_runtime::transaction_validity::TransactionSource,
        tx: <Block as BlockT>::Extrinsic,
        block_hash: <Block as BlockT>::Hash,
    ) -> TransactionValidity {
        log::debug!(target: LOG_TARGET,"Entering validate_transaction. tx: {:?}", tx);
        // Self::do_validate_transaction(source, tx, block_hash)
        Self::do_validate_transaction(tx, block_hash)
    }
}

impl frame_system_rpc_runtime_api::AccountNonceApi<Block, interface::AccountId, interface::Nonce> for Runtime {
    fn account_nonce(_account: interface::AccountId) -> interface::Nonce {
        Default::default()
    }
}


impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
    fn build_state(_json: Vec<u8>) -> sp_genesis_builder::Result {
        let genesis = serde_json::json!({});
        let _ = serde_json::to_string(&genesis)
            .expect("genesis state should be convertible to json")
            .into_bytes();
        #[cfg(feature = "std")]
        sp_io::storage::set(well_known_keys::CODE, &WASM_BINARY.unwrap().to_vec());
        Ok(())

    }

    fn get_preset(id: &Option<PresetId>) -> Option<Vec<u8>> {
        let patch = match id.as_ref() {
            Some(id) if id == sp_genesis_builder::DEV_RUNTIME_PRESET => genesis_config_presets::development_config_genesis(),
            _ =>  genesis_config_presets::development_config_genesis(),
        };
        Some(
            serde_json::to_string(&patch)
                .expect("serialization to json is expected to work. qed.")
                .into_bytes(),
        )
    }

    fn preset_names() -> Vec<PresetId> {
        vec![PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET), PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET)]

    }
}

impl sp_api::Metadata<Block> for Runtime {
    fn metadata() -> OpaqueMetadata {
        OpaqueMetadata::new(Default::default())
    }

    fn metadata_at_version(_version: u32) -> Option<OpaqueMetadata> {
        Default::default()
    }

    fn metadata_versions() -> sp_std::vec::Vec<u32> {
        Default::default()
    }
}

impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
    fn offchain_worker(_header: &<Block as BlockT>::Header) {}
}

impl sp_session::SessionKeys<Block> for Runtime {
    fn generate_session_keys(_: Option<Vec<u8>>) -> Vec<u8> {
        Default::default()
    }

    fn decode_session_keys(_: Vec<u8>) -> Option<Vec<(Vec<u8>, sp_core::crypto::KeyTypeId)>> {
        Default::default()
    }
}
}

/// Some re-exports that the node side code needs to know. Some are useful in this context as well.
///
/// Other types should preferably be private.
// TODO: this should be standardized in some way, see:
// https://github.com/paritytech/substrate/issues/10579#issuecomment-1600537558
pub mod interface {
    // use super::Runtime;

    pub type Block = super::Block;
    pub use super::opaque::Block as OpaqueBlock;
    pub type AccountId = super::AccountId;
    pub type Nonce = super::Nonce;
    pub type Hash = super::Hash;
    pub type Balance = super::Balance;
    // pub type MinimumBalance = super::MinimumBalance;
}

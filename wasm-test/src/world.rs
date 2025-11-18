use std::cell::{OnceCell, RefCell};
use std::sync::Arc;
use base64::Engine;
use move_binary_format::CompiledModule;
use move_core_types::account_address::AccountAddress;
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::TypeTag;
use prometheus::Registry;
use wasi::cli::stdout::get_stdout;
use sui_execution::Executor;
use sui_protocol_config::{Chain, ProtocolConfig, ProtocolVersion};
use sui_types::base_types::{ObjectID, SequenceNumber, SuiAddress, TransactionDigest};
use sui_types::gas::SuiGasStatus;
use sui_types::in_memory_storage::InMemoryStorage;
use sui_types::metrics::LimitsMetrics;
use sui_types::object::Object;
use sui_types::programmable_transaction_builder::ProgrammableTransactionBuilder;
use sui_types::SUI_FRAMEWORK_PACKAGE_ID;
use sui_types::transaction::{Argument, CheckedInputObjects, GasData, InputObjects, ObjectReadResult, TransactionKind};
use crate::{MOVE_STD_MODULES_BYTES, SUI_FRAMEWORK_MODULES_BYTES};

thread_local! {
    pub static WORLD: OnceCell<RefCell<World>> = OnceCell::new();
}

pub struct World {
    pub protocol_config: ProtocolConfig,
    pub executor: Arc<dyn Executor + Send + Sync>,
    pub store: InMemoryStorage,
    pub registry: Registry,
    pub metrics: Arc<LimitsMetrics>,
}

pub struct Init;

wit_bindgen::generate!({
    world: "init",
});

export!(Init);

impl Guest for Init {
    fn init() -> () {
        let mut out = get_stdout();
        out.write(b"Hello from WASI Preview 2!\n").unwrap();

        let protocol_config = ProtocolConfig::get_for_version(ProtocolVersion::MAX, Chain::Mainnet);

        let executor = sui_execution::executor(&protocol_config, false).expect("Failed to init sui execution context");

        let mut store = InMemoryStorage::new(Vec::new());

        let move_std_package_modules_bytes = MOVE_STD_MODULES_BYTES.map(|bytes| base64::engine::general_purpose::STANDARD.decode(bytes).unwrap());
        let move_std_package_modules = move_std_package_modules_bytes.map(|bytes| CompiledModule::deserialize_with_config(&bytes, &protocol_config.binary_config(None)).unwrap());
        let move_std_package = Object::new_system_package(
            &move_std_package_modules,
            SequenceNumber::from_u64(0),
            vec![ObjectID::from_address(AccountAddress::from_suffix(0x1))],
            TransactionDigest::random()
        );

        let sui_package_modules_bytes = SUI_FRAMEWORK_MODULES_BYTES.map(|bytes| base64::engine::general_purpose::STANDARD.decode(bytes).unwrap());
        let sui_package_modules = sui_package_modules_bytes.map(|bytes| CompiledModule::deserialize_with_config(&bytes, &protocol_config.binary_config(None)).unwrap());
        let sui_package = Object::new_system_package(
            &sui_package_modules,
            SequenceNumber::from_u64(0),
            vec![ObjectID::from_address(AccountAddress::from_suffix(0x1))],
            TransactionDigest::random()
        );

        store.insert_object(move_std_package);
        store.insert_object(sui_package);

        let registry = prometheus::Registry::new();
        let metrics = Arc::new(LimitsMetrics::new(&registry));

        WORLD.with(|s| s.set(
            RefCell::new(
                World {
                    protocol_config,
                    executor,
                    store,
                    registry,
                    metrics,
                }
            )
        )).unwrap_or_else(|_| panic!("Init error"));
    }
}


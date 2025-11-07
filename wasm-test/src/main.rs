use std::sync::Arc;
use wasi::cli::stdout::get_stdout;
use simulacrum::Simulacrum;
use sui_protocol_config::{Chain, ProtocolConfig, ProtocolVersion};
use sui_types::base_types::{EpochId, ObjectRef, SuiAddress, TransactionDigest};
use sui_types::crypto::{AccountKeyPair, SuiKeyPair};
use sui_types::effects::TransactionEffectsV2;
use sui_types::execution_params::ExecutionOrEarlyError;
use sui_types::gas::SuiGasStatus;
use sui_types::in_memory_storage::InMemoryStorage;
use sui_types::metrics::LimitsMetrics;
use sui_types::object::Object;
use sui_types::transaction::{Argument, CallArg, CheckedInputObjects, Command, GasData, InputObjects, ObjectArg, ObjectReadResult, ProgrammableTransaction, Transaction, TransactionKind};

fn main() {
    // Get stdout from the WASI CLI environment
    let mut out = get_stdout();
    out.write(b"Hello from WASI Preview 2!\n").unwrap();

    let protocol_config = ProtocolConfig::get_for_version(ProtocolVersion::MAX, Chain::Mainnet);

    let executor = sui_execution::executor(&protocol_config, false).expect("Failed to init sui execution context");

    let mut store = InMemoryStorage::new(Vec::new());
    let gas_coin = Object::new_gas_with_balance_and_owner_for_testing(100000000000u64, SuiAddress::default());
    let gas_coin_ref = gas_coin.compute_object_reference();

    let registry = prometheus::Registry::new();
    let metrics = Arc::new(LimitsMetrics::new(&registry));
    let gas_data = GasData {
        payment: vec![gas_coin_ref],
        owner: Default::default(),
        price: 10,
        budget: 1000000000,
    };
    let gas_status = SuiGasStatus::new_unmetered();

    let ptb = ProgrammableTransaction {
        inputs: vec![
            CallArg::Pure(vec![0, 0, 0, 0, 0, 0, 0, 7])
        ],
        commands: vec![
            Command::SplitCoins(Argument::GasCoin, vec![Argument::Input(0)])
        ],
    };

    let result = executor.execute_transaction_to_effects(
        &store,
        &protocol_config,
        metrics,
        false,
        Ok(()),
        &100,
        1000000,
        CheckedInputObjects::new_for_replay(InputObjects::new(vec![ObjectReadResult::new_from_gas_object(&gas_coin)])),
        gas_data,
        gas_status,
        TransactionKind::ProgrammableTransaction(ptb),
        SuiAddress::default(),
        TransactionDigest::random(),
        &mut None
    );

    let result_str = format!("{:?}", result.2);

    out.write(&result_str.as_bytes()).unwrap();
}

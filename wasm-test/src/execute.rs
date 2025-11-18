use crate::world::{World, WORLD};
use base64::Engine;
use std::ops::Deref;
use std::str::FromStr;
use sui_types::base_types::{SuiAddress, TransactionDigest};
use sui_types::effects::TransactionEffectsAPI;
use sui_types::gas::SuiGasStatus;
use sui_types::object::Object;
use sui_types::transaction::{CheckedInputObjects, GasData, InputObjects, ObjectReadResult, ProgrammableTransaction, TransactionKind};

wit_bindgen::generate!({
    world: "execute",
});
export!(Execute);

pub struct Execute;

impl Guest for Execute {
    fn execute(transaction_kind: Vec<u8>) -> TransactionEffects {
        WORLD.with(|world| execute_with_world(&world.get().expect("not initialized").borrow(), transaction_kind))
    }
}

fn execute_with_world(world: &World, transaction_kind: Vec<u8>) -> TransactionEffects {
    let gas_coin = Object::new_gas_with_balance_and_owner_for_testing(100000000000u64, SuiAddress::default());
    let gas_coin_ref = gas_coin.compute_object_reference();

    let gas_budget = 1000000000;
    let gas_price = 10;
    let gas_data = GasData {
        payment: vec![gas_coin_ref],
        owner: Default::default(),
        price: gas_price,
        budget: gas_budget,
    };
    let gas_status = SuiGasStatus::new(
        gas_budget,
        gas_price,
        gas_price,
        &world.protocol_config
    ).unwrap();

    let transaction_kind = bcs::from_bytes::<TransactionKind>(&transaction_kind).unwrap();
    let TransactionKind::ProgrammableTransaction(ptb) = transaction_kind else {
        panic!("Only PTBs are supported.")
    };

    let result = world.executor.execute_transaction_to_effects(
        &world.store,
        &world.protocol_config,
        world.metrics.clone(),
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

    let status = match result.2.status() {
        sui_types::execution_status::ExecutionStatus::Success => TransactionStatus::Success,
        sui_types::execution_status::ExecutionStatus::Failure { error, command } => TransactionStatus::Failure(
            TransactionStatusFailure {
                error: error.to_string(),
                command: command.map(|c| usize::from(c) as u32)
            }
        )
    };

    let object_changes = result.2.object_changes()
        .into_iter()
        .map(|object_change| {
            let id_operation = match object_change.id_operation {
                sui_types::effects::IDOperation::None => IdOperation::None,
                sui_types::effects::IDOperation::Created => IdOperation::Created,
                sui_types::effects::IDOperation::Deleted => IdOperation::Deleted,
            };

            TransactionObjectChange {
                id: object_change.id.to_vec(),
                input_version: object_change.input_version.map(Into::into),
                input_digest: object_change.input_digest.map(|d| d.inner().to_vec()),
                output_version: object_change.output_version.map(Into::into),
                output_digest: object_change.output_digest.map(|d| d.inner().to_vec()),
                id_operation
            }
        })
        .collect();

    TransactionEffects {
        executed_epoch: result.2.executed_epoch().into(),
        status,
        object_changes
    }
}
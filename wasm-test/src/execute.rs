use crate::world::{WORLD, World};
use base64::Engine;
use std::ops::Deref;
use std::str::FromStr;
use sui_types::base_types::{SuiAddress, TransactionDigest};
use sui_types::effects::TransactionEffectsAPI;
use sui_types::gas::SuiGasStatus;
use sui_types::object::Object;
use sui_types::transaction::{
    CheckedInputObjects, GasData, InputObjects, ObjectReadResult, ProgrammableTransaction,
    Transaction, TransactionData, TransactionKind,
};

wit_bindgen::generate!({
    world: "execute",
});
export!(Execute);

pub struct Execute;

impl Guest for Execute {
    fn execute(transaction: Vec<u8>) -> TransactionEffects {
        WORLD.with(|world| {
            let mut world_ref = world.get().expect("not initialized").borrow_mut();
            execute_with_world(&mut world_ref, transaction)
        })
    }
}

fn execute_with_world(world: &mut World, transaction: Vec<u8>) -> TransactionEffects {
    let transaction = bcs::from_bytes::<TransactionData>(&transaction).unwrap();
    let (transaction_kind, sender, gas_data) = transaction.execution_parts();
    let TransactionKind::ProgrammableTransaction(ptb) = transaction_kind else {
        panic!("Only PTBs are supported.")
    };

    let gas_status = SuiGasStatus::new(
        gas_data.budget,
        gas_data.price,
        gas_data.price,
        &world.protocol_config,
    )
    .unwrap();

    let gas_object_read_results = gas_data
        .payment
        .iter()
        .map(|gas| {
            let gas_object = world.store.get_object(&gas.0).unwrap();
            ObjectReadResult::new_from_gas_object(gas_object)
        })
        .collect();

    let gas_checked_inputs =
        CheckedInputObjects::new_for_replay(InputObjects::new(gas_object_read_results));

    let (inner_temp_store, _, effects, _, _) = world.executor.execute_transaction_to_effects(
        &world.store,
        &world.protocol_config,
        world.metrics.clone(),
        false,
        Ok(()),
        &100,
        1000000,
        gas_checked_inputs,
        gas_data,
        gas_status,
        TransactionKind::ProgrammableTransaction(ptb),
        SuiAddress::default(),
        TransactionDigest::random(),
        &mut None,
    );

    let should_commit = matches!(
        effects.status(),
        sui_types::execution_status::ExecutionStatus::Success
    );

    if should_commit {
        effects
            .deleted()
            .into_iter()
            .chain(effects.unwrapped_then_deleted())
            .chain(effects.wrapped())
            .for_each(|(object_id, _, _)| {
                world.store.remove_object(object_id);
            });
        world.store.finish(inner_temp_store.written);
    }

    let status = match effects.status() {
        sui_types::execution_status::ExecutionStatus::Success => TransactionStatus::Success,
        sui_types::execution_status::ExecutionStatus::Failure { error, command } => {
            TransactionStatus::Failure(TransactionStatusFailure {
                error: error.to_string(),
                command: command.map(|c| usize::from(c) as u32),
            })
        }
    };

    let object_changes = effects
        .object_changes()
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
                id_operation,
            }
        })
        .collect();

    TransactionEffects {
        executed_epoch: effects.executed_epoch().into(),
        status,
        object_changes,
    }
}

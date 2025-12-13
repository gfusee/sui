use crate::world::{WORLD, World};
use sui_types::base_types::{ObjectRef, SuiAddress, TransactionDigest};
use sui_types::effects::{TransactionEffects, TransactionEffectsAPI};
use sui_types::effects::ObjectRemoveKind;
use sui_types::gas::SuiGasStatus;
use sui_types::inner_temporary_store::PackageStoreWithFallback;
use sui_types::object::{MoveObject, Object, OBJECT_START_VERSION};
use sui_types::object::Owner;
use sui_types::transaction::{CallArg, CheckedInputObjects, GasData, InputObjectKind, InputObjects, ObjectReadResult, ObjectReadResultKind, ProgrammableTransaction, Transaction, TransactionData, TransactionKind};
use sui_types::{base_types::ObjectID, base_types::SequenceNumber};
use sui_types::storage::{ObjectStore, WriteKind};

wit_bindgen::generate!({
    world: "execute",
});
export!(Execute);

pub struct Execute;

impl Guest for Execute {
    fn execute(transaction: Vec<u8>) -> SuiTransactionBlockResponse {
        WORLD.with(|world| {
            let mut world_ref = world.get().expect("not initialized").borrow_mut();
            execute_with_world(&mut world_ref, transaction)
        })
    }

    fn dry_run_execute(transaction: Vec<u8>) -> SuiTransactionBlockResponse {
        WORLD.with(|world| {
            let mut world_ref = world.get().expect("not initialized").borrow_mut();
            dry_execute_with_world(&mut world_ref, transaction)
        })
    }
}

struct ExecutionOutput {
    inner_temp_store: sui_types::inner_temporary_store::InnerTemporaryStore,
    effects: TransactionEffects,
    tx_digest: TransactionDigest,
    status: TransactionStatus,
    detailed_object_changes: Vec<ObjectChange>,
    raw_effects: Vec<u8>,
    block_effects: SuiTransactionBlockEffects,
}

type DeserializedProgrammableTransaction = (ProgrammableTransaction, SuiAddress, GasData);

fn execute_with_world_common(
    world: &mut World,
    (ptb, sender, mut gas_data): DeserializedProgrammableTransaction,
    override_gas_payment: Option<Vec<Object>>,
) -> ExecutionOutput {
    let gas_status = SuiGasStatus::new(
        gas_data.budget,
        gas_data.price,
        gas_data.price,
        &world.protocol_config,
    )
    .unwrap();

    let gas_data_payment: Vec<_> = if let Some(override_gas_payment) = override_gas_payment {
        override_gas_payment
            .iter()
            .map(|object| {
                ObjectReadResult::new_from_gas_object(&object)
            })
            .collect()
    } else {
        gas_data.payment
            .iter()
            .map(|object_ref| {
                let object = world.store.get_object(&object_ref.0).unwrap();

                ObjectReadResult::new_from_gas_object(&object)
            })
            .collect()
    };

    let mut non_gas_input_objects = ptb.inputs
        .iter()
        .filter_map(|call_arg| {
            let CallArg::Object(obj) = call_arg else {
                return None
            };

            let object = world.store.get_object(&obj.id()).unwrap();

            let result = ObjectReadResult::new(
                InputObjectKind::ImmOrOwnedMoveObject(object.compute_object_reference()),// TODO: add support for shared objects
                ObjectReadResultKind::Object(object.clone())
            );

            Some(result)
        })
        .collect();

    let mut input_objects = gas_data_payment;
    input_objects.append(&mut non_gas_input_objects);

    let checked_inputs =
        CheckedInputObjects::new_with_checked_transaction_inputs(InputObjects::new(input_objects));

    let tx_digest = TransactionDigest::random();

    let (inner_temp_store, _, effects, _, _) = world.executor.execute_transaction_to_effects(
        &world.store,
        &world.protocol_config,
        world.metrics.clone(),
        false,
        Ok(()),
        &100,
        1000000,
        checked_inputs,
        gas_data,
        gas_status,
        TransactionKind::ProgrammableTransaction(ptb),
        SuiAddress::default(),
        tx_digest,
        &mut None,
    );

    {
        let package_store = PackageStoreWithFallback::new(&inner_temp_store, &world.store);
        let _layout_resolver = world
            .executor
            .type_layout_resolver(Box::new(package_store));
        let _ = &_layout_resolver;
    }

    let _transaction_events = inner_temp_store.events.clone();
    let detailed_object_changes = build_object_changes(
        &inner_temp_store,
        &world.store,
        &effects,
        sender,
    );

    let status = match effects.status() {
        sui_types::execution_status::ExecutionStatus::Success => TransactionStatus::Success,
        sui_types::execution_status::ExecutionStatus::Failure { error, command } => {
            TransactionStatus::Failure(TransactionStatusFailure {
                error: error.to_string(),
                command: command.map(|c| usize::from(c) as u32),
            })
        }
    };

    let block_effects = to_wit_transaction_block_effects(&effects, status.clone(), tx_digest);
    let raw_effects = bcs::to_bytes(&effects).unwrap_or_default();

    ExecutionOutput {
        inner_temp_store,
        effects,
        tx_digest,
        status,
        detailed_object_changes,
        raw_effects,
        block_effects,
    }
}

fn execute_with_world(
    world: &mut World,
    transaction: Vec<u8>,
) -> SuiTransactionBlockResponse {
    let output = execute_with_world_common(
        world,
        deserialize_transaction(transaction),
        None
    );

    let should_commit = matches!(&output.status, TransactionStatus::Success);

    if should_commit {
        output
            .effects
            .deleted()
            .into_iter()
            .chain(output.effects.unwrapped_then_deleted())
            .chain(output.effects.wrapped())
            .for_each(|(object_id, _, _)| {
                world.store.remove_object(object_id);
            });
        world.store.finish(output.inner_temp_store.written);
    }

    SuiTransactionBlockResponse {
        digest: output.tx_digest.into_inner().to_vec(),
        effects: Some(output.block_effects),
        object_changes: output.detailed_object_changes,
        raw_effects: output.raw_effects,
    }
}

fn dry_execute_with_world(
    world: &mut World,
    transaction: Vec<u8>,
) -> SuiTransactionBlockResponse {
    let mut deserialized_transaction = deserialize_transaction(transaction);

    let override_gas_payment = if deserialized_transaction.2.payment.is_empty() {
        let sender = deserialized_transaction.1;

        const MIST_TO_SUI: u64 = 1_000_000_000;
        const DRY_RUN_SUI: u64 = 1_000_000_000;
        let max_coin_value = MIST_TO_SUI * DRY_RUN_SUI;
        let gas_object_id = ObjectID::random();
        let gas_object = Object::new_move(
            MoveObject::new_gas_coin(OBJECT_START_VERSION, gas_object_id, max_coin_value),
            Owner::AddressOwner(sender),
            TransactionDigest::genesis_marker(),
        );

        Some(vec![gas_object])
    } else {
        None
    };

    let output = execute_with_world_common(
        world,
        deserialized_transaction,
        override_gas_payment
    );

    SuiTransactionBlockResponse {
        digest: output.tx_digest.into_inner().to_vec(),
        effects: Some(output.block_effects),
        object_changes: output.detailed_object_changes,
        raw_effects: output.raw_effects,
    }
}

fn deserialize_transaction(transaction: Vec<u8>) -> (ProgrammableTransaction, SuiAddress, GasData) {
    let transaction = bcs::from_bytes::<TransactionData>(&transaction).unwrap();
    let (transaction_kind, sender, gas_data) = transaction.execution_parts();
    let TransactionKind::ProgrammableTransaction(ptb) = transaction_kind else {
        panic!("Only PTBs are supported.")
    };

    (ptb, sender, gas_data)
}

fn build_object_changes(
    temp_store: &sui_types::inner_temporary_store::InnerTemporaryStore,
    base_store: &sui_types::in_memory_storage::InMemoryStorage,
    effects: &impl TransactionEffectsAPI,
    sender: SuiAddress,
) -> Vec<ObjectChange> {
    let provider = LocalObjectProvider {
        written: &temp_store.written,
        input_objects: &temp_store.input_objects,
        base_store,
    };

    let modified_at_versions = effects
        .modified_at_versions()
        .into_iter()
        .collect::<std::collections::BTreeMap<ObjectID, SequenceNumber>>();

    let mut changes = Vec::new();

    let changed_objects = effects
        .created()
        .into_iter()
        .map(|entry| (entry, WriteKind::Create))
        .chain(
            effects
                .mutated()
                .into_iter()
                .map(|entry| (entry, WriteKind::Mutate)),
        )
        .chain(
            effects
                .unwrapped()
                .into_iter()
                .map(|entry| (entry, WriteKind::Unwrap)),
        );

    for ((object_ref, owner), write_kind) in changed_objects {
        let (object_id, version, digest) = object_ref;
        if let Some(object) = provider.get_object(&object_id, version) {
            if object.is_package() && matches!(write_kind, WriteKind::Create) {
                let package = object.data.try_as_package().unwrap();
                let modules = package.serialized_module_map().keys().cloned().collect();
                changes.push(ObjectChange::Published(ObjectChangePublished {
                    package_id: package.id().into_bytes().to_vec(),
                    version: package.version().value(),
                    digest: digest.into_inner().to_vec(),
                    modules,
                }));
                continue;
            }

            if let Some(object_type) = object.type_() {
                match write_kind {
                    WriteKind::Mutate => {
                        let previous_version = *modified_at_versions
                            .get(&object_id)
                            .unwrap_or(&SequenceNumber::default());
                        changes.push(ObjectChange::Mutated(ObjectChangeMutated {
                            sender: sender.to_vec(),
                            owner: to_ownership(owner),
                            object_type: object_type.to_string(),
                            object_id: object_id.into_bytes().to_vec(),
                            version: version.value(),
                            previous_version: previous_version.value(),
                            digest: digest.into_inner().to_vec(),
                        }));
                    }
                    WriteKind::Create => {
                        changes.push(ObjectChange::Created(ObjectChangeCreated {
                            sender: sender.to_vec(),
                            owner: to_ownership(owner),
                            object_type: object_type.to_string(),
                            object_id: object_id.into_bytes().to_vec(),
                            version: version.value(),
                            digest: digest.into_inner().to_vec(),
                        }));
                    }
                    WriteKind::Unwrap => {
                        changes.push(ObjectChange::Created(ObjectChangeCreated {
                            sender: sender.to_vec(),
                            owner: to_ownership(owner),
                            object_type: object_type.to_string(),
                            object_id: object_id.into_bytes().to_vec(),
                            version: version.value(),
                            digest: digest.into_inner().to_vec(),
                        }));
                    }
                }
            }
        }
    }

    let removed_objects = effects
        .deleted()
        .into_iter()
        .map(|oref| (oref, ObjectRemoveKind::Delete))
        .chain(
            effects
                .unwrapped_then_deleted()
                .into_iter()
                .map(|oref| (oref, ObjectRemoveKind::Delete)),
        )
        .chain(
            effects
                .wrapped()
                .into_iter()
                .map(|oref| (oref, ObjectRemoveKind::Wrap)),
        );

    for ((object_id, version, _digest), remove_kind) in removed_objects {
        if let Some(object) = provider.find_object_le(&object_id, version) {
            if let Some(object_type) = object.type_() {
                match remove_kind {
                    ObjectRemoveKind::Delete => changes.push(ObjectChange::Deleted(
                        ObjectChangeDeleted {
                            sender: sender.to_vec(),
                            object_type: object_type.to_string(),
                            object_id: object_id.into_bytes().to_vec(),
                            version: version.value(),
                        },
                    )),
                    ObjectRemoveKind::Wrap => changes.push(ObjectChange::Wrapped(
                        ObjectChangeWrapped {
                            sender: sender.to_vec(),
                            object_type: object_type.to_string(),
                            object_id: object_id.into_bytes().to_vec(),
                            version: version.value(),
                        },
                    )),
                }
            }
        }
    }

    changes
}

struct LocalObjectProvider<'a> {
    written: &'a sui_types::inner_temporary_store::WrittenObjects,
    input_objects: &'a std::collections::BTreeMap<ObjectID, Object>,
    base_store: &'a sui_types::in_memory_storage::InMemoryStorage,
}

impl LocalObjectProvider<'_> {
    fn get_object(&self, id: &ObjectID, version: SequenceNumber) -> Option<Object> {
        if let Some(obj) = self.written.get(id) {
            if obj.version() == version {
                return Some(obj.clone());
            }
        }

        if let Some(obj) = self.input_objects.get(id) {
            if obj.version() == version {
                return Some(obj.clone());
            }
        }

        self.base_store.get_object_by_key(id, version)
    }

    fn find_object_le(&self, id: &ObjectID, version: SequenceNumber) -> Option<Object> {
        if let Some(obj) = self.written.get(id) {
            if obj.version() <= version {
                return Some(obj.clone());
            }
        }

        if let Some(obj) = self.input_objects.get(id) {
            if obj.version() <= version {
                return Some(obj.clone());
            }
        }

        self.base_store.get_object_by_key(id, version)
    }
}

fn to_wit_transaction_block_effects(
    effects: &impl TransactionEffectsAPI,
    status: TransactionStatus,
    tx_digest: TransactionDigest,
) -> SuiTransactionBlockEffects {
    SuiTransactionBlockEffects {
        status,
        executed_epoch: effects.executed_epoch().into(),
        gas_used: GasCostSummary {
            computation_cost: effects.gas_cost_summary().computation_cost,
            storage_cost: effects.gas_cost_summary().storage_cost,
            storage_rebate: effects.gas_cost_summary().storage_rebate,
            non_refundable_storage_fee: effects.gas_cost_summary().non_refundable_storage_fee,
        },
        transaction_digest: tx_digest.into_inner().to_vec(),
        created: effects
            .created()
            .into_iter()
            .map(to_owned_object_ref)
            .collect(),
        mutated: effects
            .mutated()
            .into_iter()
            .map(to_owned_object_ref)
            .collect(),
        unwrapped: effects
            .unwrapped()
            .into_iter()
            .map(to_owned_object_ref)
            .collect(),
        deleted: effects
            .deleted()
            .into_iter()
            .map(to_sui_object_ref)
            .collect(),
        unwrapped_then_deleted: effects
            .unwrapped_then_deleted()
            .into_iter()
            .map(to_sui_object_ref)
            .collect(),
        wrapped: effects
            .wrapped()
            .into_iter()
            .map(to_sui_object_ref)
            .collect(),
        gas_object: to_owned_object_ref(effects.gas_object()),
        dependencies: effects
            .dependencies()
            .iter()
            .map(|d| d.into_inner().to_vec())
            .collect(),
    }
}

fn to_owned_object_ref((object_ref, owner): (ObjectRef, Owner)) -> OwnedObjectRef {
    OwnedObjectRef {
        reference: to_sui_object_ref(object_ref),
        owner: to_ownership(owner),
    }
}

fn to_sui_object_ref((object_id, version, digest): ObjectRef) -> SuiObjectRef {
    SuiObjectRef {
        object_id: object_id.into_bytes().to_vec(),
        version: version.value(),
        digest: digest.into_inner().to_vec(),
    }
}

fn to_ownership(owner: Owner) -> Ownership {
    match owner {
        Owner::AddressOwner(addr) => Ownership::AddressOwner(addr.to_vec()),
        Owner::ObjectOwner(id) => Ownership::ObjectOwner(id.to_vec()),
        Owner::Shared {
            initial_shared_version,
        } => Ownership::Shared(SharedOwnership {
            initial_shared_version: initial_shared_version.value(),
        }),
        Owner::Immutable => Ownership::Immutable,
        Owner::ConsensusAddressOwner { start_version, owner } => Ownership::ConsensusAddress(
            ConsensusAddressOwnership {
                start_version: start_version.value(),
                owner: owner.to_vec(),
            },
        ),
    }
}

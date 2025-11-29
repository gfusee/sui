use crate::world::{World, WORLD};
use std::convert::TryFrom;
use sui_types::base_types::{ObjectID, SuiAddress};
use sui_types::move_package::{MovePackage, TypeOrigin as MoveTypeOrigin, UpgradeInfo};
use sui_types::object::{Data, MoveObject, Object, Owner};

wit_bindgen::generate!({
    world: "client",
});
export!(Client);

pub struct Client;

impl Guest for Client {
    fn get_objects_for_address(address: Vec<u8>) -> Vec<SuiObject> {
        let address =
            SuiAddress::try_from(address.as_slice()).expect("address must be 32 bytes long");

        WORLD.with(|world| {
            let world_ref = world.get().expect("not initialized").borrow();
            get_objects_for_address(&world_ref, address)
        })
    }

    fn get_object(id: Vec<u8>) -> Option<SuiObject> {
        let id = ObjectID::try_from(id.as_slice()).expect("object id must be 32 bytes long");
        WORLD.with(|world| {
            world
                .get()
                .expect("not initialized")
                .borrow()
                .store
                .objects()
                .get(&id)
                .map(to_sui_object)
        })
    }

    fn faucet(recipient: Vec<u8>, amount: u64) -> SuiObject {
        let recipient =
            SuiAddress::try_from(recipient.as_slice()).expect("recipient must be 32 bytes long");

        WORLD.with(|world| {
            let mut world_ref = world.get().expect("not initialized").borrow_mut();
            let object = Object::new_gas_with_balance_and_owner_for_testing(amount, recipient);
            let owned = to_sui_object(&object);
            world_ref.store.insert_object(object);
            owned
        })
    }
}

fn get_objects_for_address(world: &World, address: SuiAddress) -> Vec<SuiObject> {
    world
        .store
        .objects()
        .iter()
        .filter_map(|(_, object)| match &object.owner {
            Owner::AddressOwner(owner) if owner == &address => Some(to_sui_object(object)),
            _ => None,
        })
        .collect()
}

fn to_sui_object(object: &Object) -> SuiObject {
    SuiObject {
        id: object.id().into_bytes().to_vec(),
        version: object.version().value(),
        digest: object.digest().inner().to_vec(),
        data: to_object_data(&object.data),
        owner: to_ownership(&object.owner),
        previous_transaction: object.previous_transaction.into_inner().to_vec(),
        storage_rebate: object.storage_rebate,
    }
}

fn to_object_data(data: &Data) -> ObjectData {
    match data {
        Data::Move(move_obj) => ObjectData::MoveObject(to_move_object_data(move_obj)),
        Data::Package(pkg) => ObjectData::MovePackage(to_package_object_data(pkg)),
    }
}

fn to_move_object_data(object: &MoveObject) -> MoveObjectData {
    MoveObjectData {
        type_bcs: bcs::to_bytes(object.type_()).expect("serialize move object type"),
        type_repr: object.type_().to_string(),
        has_public_transfer: object.has_public_transfer(),
        version: object.version().value(),
        contents: object.contents().to_vec(),
    }
}

fn to_package_object_data(package: &MovePackage) -> PackageObjectData {
    PackageObjectData {
        id: package.id().into_bytes().to_vec(),
        version: package.version().value(),
        modules: package
            .serialized_module_map()
            .iter()
            .map(|(name, bytes)| ModuleBytes {
                name: name.clone(),
                bytes: bytes.clone(),
            })
            .collect(),
        type_origin_table: package
            .type_origin_table()
            .iter()
            .map(to_package_type_origin)
            .collect(),
        linkage_table: package
            .linkage_table()
            .iter()
            .map(to_package_link)
            .collect(),
    }
}

fn to_package_type_origin(origin: &MoveTypeOrigin) -> PackageTypeOrigin {
    PackageTypeOrigin {
        module_name: origin.module_name.clone(),
        datatype_name: origin.datatype_name.clone(),
        package_id: origin.package.into_bytes().to_vec(),
    }
}

fn to_package_link((original_package, info): (&ObjectID, &UpgradeInfo)) -> PackageLink {
    PackageLink {
        original_package_id: original_package.into_bytes().to_vec(),
        upgraded_id: info.upgraded_id.into_bytes().to_vec(),
        upgraded_version: info.upgraded_version.value(),
    }
}

fn to_ownership(owner: &Owner) -> Ownership {
    match owner {
        Owner::AddressOwner(address) => Ownership::AddressOwner(address.to_vec()),
        Owner::ObjectOwner(object_id) => Ownership::ObjectOwner(object_id.to_vec()),
        Owner::Shared {
            initial_shared_version,
        } => Ownership::Shared(SharedOwnership {
            initial_shared_version: initial_shared_version.value(),
        }),
        Owner::Immutable => Ownership::Immutable,
        Owner::ConsensusAddressOwner {
            start_version,
            owner,
        } => Ownership::Consensus(ConsensusOwnership {
            start_version: start_version.value(),
            owner: owner.to_vec(),
        }),
    }
}

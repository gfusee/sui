use crate::sui_move_json_rpc::{
    SuiMoveAbility, SuiMoveAbilitySet, SuiMoveNormalizedEnum, SuiMoveNormalizedField,
    SuiMoveNormalizedFunction, SuiMoveNormalizedModule, SuiMoveNormalizedStruct,
    SuiMoveNormalizedStructType, SuiMoveNormalizedType, SuiMoveVisibility,
};
use move_binary_format::{binary_config::BinaryConfig, normalized};
use sui_types::move_package::normalize_modules;

wit_bindgen::generate!({
    world: "move-package",
});
export!(Package);

pub struct Package;

impl Guest for Package {
    fn get_normalized_package(modules: Vec<Vec<u8>>) -> Vec<NormalizedModule> {
        let mut pool = normalized::RcPool::new();
        let binary_config = BinaryConfig::legacy_with_flags(false, false);
        let normalized = normalize_modules(&mut pool, modules.iter(), &binary_config, false)
            .expect("normalizing modules cannot fail");

        normalized
            .into_iter()
            .map(|(name, module)| {
                let normalized_module: SuiMoveNormalizedModule = (&module).into();
                (name, normalized_module)
            })
            .map(|(name, module)| NormalizedModule {
                file_format_version: module.file_format_version,
                address: module.address,
                name,
                friends: module
                    .friends
                    .into_iter()
                    .map(|f| NormalizedModuleId {
                        address: f.address,
                        name: f.name,
                    })
                    .collect(),
                structs: module
                    .structs
                    .into_iter()
                    .map(|(name, data)| NormalizedModulePair {
                        name,
                        data: to_normalized_struct(data),
                    })
                    .collect(),
                enums: module
                    .enums
                    .into_iter()
                    .map(|(name, data)| NormalizedModuleEnumPair {
                        name,
                        data: to_normalized_enum(data),
                    })
                    .collect(),
                exposed_functions: module
                    .exposed_functions
                    .into_iter()
                    .map(|(name, data)| NormalizedModuleFunctionPair {
                        name,
                        data: to_normalized_function(data),
                    })
                    .collect(),
            })
            .collect()
    }
}

fn to_normalized_struct(value: SuiMoveNormalizedStruct) -> NormalizedStruct {
    NormalizedStruct {
        abilities: to_ability_set(value.abilities),
        type_parameters: value
            .type_parameters
            .into_iter()
            .map(|param| NormalizedStructTypeParameter {
                constraints: to_ability_set(param.constraints),
                is_phantom: param.is_phantom,
            })
            .collect(),
        fields: value
            .fields
            .into_iter()
            .map(|field| NormalizedField {
                name: field.name,
                move_type: bcs::to_bytes(&field.type_).unwrap(),
            })
            .collect(),
    }
}

fn to_normalized_enum(value: SuiMoveNormalizedEnum) -> NormalizedEnum {
    NormalizedEnum {
        abilities: to_ability_set(value.abilities),
        type_parameters: value
            .type_parameters
            .into_iter()
            .map(|param| NormalizedStructTypeParameter {
                constraints: to_ability_set(param.constraints),
                is_phantom: param.is_phantom,
            })
            .collect(),
        variants: value
            .variants
            .into_iter()
            .map(|(name, fields)| NormalizedEnumVariant {
                name,
                fields: fields
                    .into_iter()
                    .map(|field| NormalizedField {
                        name: field.name,
                        move_type: bcs::to_bytes(&field.type_).unwrap(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn to_normalized_function(value: SuiMoveNormalizedFunction) -> NormalizedFunction {
    NormalizedFunction {
        visibility: match value.visibility {
            SuiMoveVisibility::Private => NormalizedVisibility::Private,
            SuiMoveVisibility::Public => NormalizedVisibility::Public,
            SuiMoveVisibility::Friend => NormalizedVisibility::Friend,
        },
        is_entry: value.is_entry,
        type_parameters: value
            .type_parameters
            .into_iter()
            .map(to_ability_set)
            .collect(),
        parameters: value
            .parameters
            .into_iter()
            .map(to_normalized_type)
            .collect(),
        return_: value.return_.into_iter().map(to_normalized_type).collect(),
    }
}

fn to_normalized_type(value: SuiMoveNormalizedType) -> Vec<u8> {
    bcs::to_bytes(&value).unwrap()
}

fn to_ability_set(value: SuiMoveAbilitySet) -> AbilitySet {
    AbilitySet {
        abilities: value
            .abilities
            .into_iter()
            .map(|ability| match ability {
                SuiMoveAbility::Copy => Ability::Copy,
                SuiMoveAbility::Drop => Ability::Drop,
                SuiMoveAbility::Store => Ability::Store,
                SuiMoveAbility::Key => Ability::Key,
            })
            .collect(),
    }
}

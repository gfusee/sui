use std::collections::BTreeMap;
use crate::sui_move_json_rpc::{
    SuiMoveAbility, SuiMoveAbilitySet, SuiMoveNormalizedEnum, SuiMoveNormalizedField,
    SuiMoveNormalizedFunction, SuiMoveNormalizedModule, SuiMoveNormalizedStruct,
    SuiMoveNormalizedStructType, SuiMoveNormalizedType, SuiMoveVisibility,
};
use move_binary_format::{binary_config::BinaryConfig, normalized};
use move_command_line_common::files::FileHash;
use move_compiler::parser::{
    ast::{
        self as ast_defs, Definition as MoveAstDefinition, LeadingNameAccess, LeadingNameAccess_,
        ModuleMember,
    },
    syntax::parse_file_string,
};
use move_compiler::shared::{CompilationEnv, Flags};
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

    fn get_definitions(move_code: String) -> Vec<MoveDefinition> {
        let ast_defs = parse_ast_definitions(&move_code);
        ast_defs
            .into_iter()
            .flat_map(convert_definition)
            .collect()
    }
}

fn parse_ast_definitions(move_code: &str) -> Vec<MoveAstDefinition> {
    let compilation_env = CompilationEnv::new(
        Flags::empty(),
        vec![],
        vec![],
        None,
        BTreeMap::new(),
        None,
        None
    );
    let file_hash = FileHash::new(move_code);

    parse_file_string(&compilation_env, file_hash, move_code, None)
        .expect("failed to parse Move code")
}

fn convert_definition(def: MoveAstDefinition) -> Option<MoveDefinition> {
    match def {
        MoveAstDefinition::Module(module) => Some(MoveDefinition::Module(convert_module(module))),
        MoveAstDefinition::Address(address) => Some(MoveDefinition::Address(MoveAddress {
            address: leading_name_to_string(&address.addr),
            modules: address.modules.into_iter().map(convert_module).collect(),
        })),
    }
}

fn convert_module(module: ast_defs::ModuleDefinition) -> MoveModule {
    let members = module
        .members
        .into_iter()
        .filter_map(convert_member)
        .collect();

    MoveModule {
        address: module.address.as_ref().map(leading_name_to_string),
        name: name_to_string(&module.name.0),
        members,
    }
}

fn convert_member(member: ModuleMember) -> Option<MoveModuleMember> {
    match member {
        ModuleMember::Function(f) => Some(MoveModuleMember {
            kind: MoveModuleMemberKind::Function,
            name: name_to_string(&f.name.0),
        }),
        ModuleMember::Struct(s) => Some(MoveModuleMember {
            kind: MoveModuleMemberKind::Struct,
            name: name_to_string(&s.name.0),
        }),
        ModuleMember::Enum(e) => Some(MoveModuleMember {
            kind: MoveModuleMemberKind::Enumeration,
            name: name_to_string(&e.name.0),
        }),
        ModuleMember::Constant(c) => Some(MoveModuleMember {
            kind: MoveModuleMemberKind::Constant,
            name: name_to_string(&c.name.0),
        }),
        ModuleMember::Use(_)
        | ModuleMember::Friend(_)
        | ModuleMember::Spec(_) => None,
    }
}

fn leading_name_to_string(addr: &LeadingNameAccess) -> String {
    match &addr.value {
        LeadingNameAccess_::AnonymousAddress(num) => format!("{num}"),
        LeadingNameAccess_::GlobalAddress(name) | LeadingNameAccess_::Name(name) => {
            name_to_string(&name)
        }
    }
}

fn name_to_string(name: &move_compiler::shared::Name) -> String {
    name.value.to_string()
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

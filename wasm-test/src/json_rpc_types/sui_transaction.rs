use move_core_types::language_storage::{StructTag, TypeTag};
use sui_types::{
    base_types::{ObjectID, SequenceNumber, SuiAddress, TransactionDigest},
    digests::{ObjectDigest, TransactionEventsDigest},
    gas::GasCostSummary,
    object::Owner,
    signature::GenericSignature,
};

pub type CheckpointSequenceNumber = u64;

// Top-level response. Field set matches sui-json-rpc SuiTransactionBlockResponse.
pub struct SuiTransactionBlockResponse {
    pub digest: TransactionDigest,
    pub transaction: Option<SuiTransactionBlock>,
    pub raw_transaction: Vec<u8>,
    pub effects: Option<SuiTransactionBlockEffects>,
    pub events: Option<SuiTransactionBlockEvents>,
    pub object_changes: Option<Vec<ObjectChange>>,
    pub balance_changes: Option<Vec<BalanceChange>>,
    pub timestamp_ms: Option<u64>,
    pub confirmed_local_execution: Option<bool>,
    pub checkpoint: Option<CheckpointSequenceNumber>,
    pub errors: Vec<String>,
    pub raw_effects: Vec<u8>,
}

// --- Effects -----------------------------------------------------------------

pub enum SuiTransactionBlockEffects {
    V1(SuiTransactionBlockEffectsV1),
}

pub trait SuiTransactionBlockEffectsAPI {
    fn status(&self) -> &SuiExecutionStatus;
    fn executed_epoch(&self) -> u64;
    fn gas_cost_summary(&self) -> &GasCostSummary;
    fn transaction_digest(&self) -> &TransactionDigest;
    fn created(&self) -> &[OwnedObjectRef];
    fn mutated(&self) -> &[OwnedObjectRef];
    fn unwrapped(&self) -> &[OwnedObjectRef];
    fn deleted(&self) -> &[SuiObjectRef];
    fn unwrapped_then_deleted(&self) -> &[SuiObjectRef];
    fn wrapped(&self) -> &[SuiObjectRef];
    fn gas_object(&self) -> &OwnedObjectRef;
    fn dependencies(&self) -> &[TransactionDigest];
}

pub struct SuiTransactionBlockEffectsModifiedAtVersions {
    pub object_id: ObjectID,
    pub sequence_number: SequenceNumber,
}

pub struct SuiTransactionBlockEffectsV1 {
    pub status: SuiExecutionStatus,
    pub executed_epoch: u64,
    pub gas_used: GasCostSummary,
    pub modified_at_versions: Vec<SuiTransactionBlockEffectsModifiedAtVersions>,
    pub shared_objects: Vec<SuiObjectRef>,
    pub transaction_digest: TransactionDigest,
    pub created: Vec<OwnedObjectRef>,
    pub mutated: Vec<OwnedObjectRef>,
    pub unwrapped: Vec<OwnedObjectRef>,
    pub deleted: Vec<SuiObjectRef>,
    pub unwrapped_then_deleted: Vec<SuiObjectRef>,
    pub wrapped: Vec<SuiObjectRef>,
    pub accumulator_events: Vec<SuiAccumulatorEvent>,
    pub gas_object: OwnedObjectRef,
    pub events_digest: Option<TransactionEventsDigest>,
    pub dependencies: Vec<TransactionDigest>,
    pub abort_error: Option<SuiMoveAbort>,
}

impl SuiTransactionBlockEffectsAPI for SuiTransactionBlockEffectsV1 {
    fn status(&self) -> &SuiExecutionStatus {
        &self.status
    }
    fn executed_epoch(&self) -> u64 {
        self.executed_epoch
    }
    fn gas_cost_summary(&self) -> &GasCostSummary {
        &self.gas_used
    }
    fn transaction_digest(&self) -> &TransactionDigest {
        &self.transaction_digest
    }
    fn created(&self) -> &[OwnedObjectRef] {
        &self.created
    }
    fn mutated(&self) -> &[OwnedObjectRef] {
        &self.mutated
    }
    fn unwrapped(&self) -> &[OwnedObjectRef] {
        &self.unwrapped
    }
    fn deleted(&self) -> &[SuiObjectRef] {
        &self.deleted
    }
    fn unwrapped_then_deleted(&self) -> &[SuiObjectRef] {
        &self.unwrapped_then_deleted
    }
    fn wrapped(&self) -> &[SuiObjectRef] {
        &self.wrapped
    }
    fn gas_object(&self) -> &OwnedObjectRef {
        &self.gas_object
    }
    fn dependencies(&self) -> &[TransactionDigest] {
        &self.dependencies
    }
}

// --- Events ------------------------------------------------------------------

pub struct SuiTransactionBlockEvents {
    pub data: Vec<SuiEvent>,
}

pub struct SuiEvent;

// --- Transactions ------------------------------------------------------------

pub enum SuiTransactionBlockData {
    V1(SuiTransactionBlockDataV1),
}

pub struct SuiTransactionBlockDataV1 {
    pub transaction: SuiTransactionBlockKind,
    pub sender: SuiAddress,
    pub gas_data: SuiGasData,
}

pub struct SuiTransactionBlock {
    pub data: SuiTransactionBlockData,
    pub tx_signatures: Vec<GenericSignature>,
}

pub enum SuiTransactionBlockKind {
    ChangeEpoch(SuiChangeEpoch),
    Genesis(SuiGenesisTransaction),
    ConsensusCommitPrologue(SuiConsensusCommitPrologue),
    ConsensusCommitPrologueV2(SuiConsensusCommitPrologueV2),
    ConsensusCommitPrologueV3(SuiConsensusCommitPrologueV3),
    ConsensusCommitPrologueV4(SuiConsensusCommitPrologueV4),
    ProgrammableTransaction(SuiProgrammableTransactionBlock),
    ProgrammableSystemTransaction(SuiProgrammableTransactionBlock),
    AuthenticatorStateUpdate(SuiAuthenticatorStateUpdate),
    RandomnessStateUpdate(SuiRandomnessStateUpdate),
    EndOfEpochTransaction(SuiEndOfEpochTransaction),
}

pub struct SuiProgrammableTransactionBlock {
    pub inputs: Vec<SuiCallArg>,
    pub commands: Vec<SuiCommand>,
}

pub enum SuiCallArg {
    Object(SuiObjectArg),
    Pure(Vec<u8>),
}

pub enum SuiObjectArg {
    ImmOrOwnedObject(SuiObjectRef),
    SharedObject {
        id: ObjectID,
        initial_shared_version: SequenceNumber,
        mutable: bool,
    },
    Receiving(SuiObjectRef),
}

pub enum SuiCommand {
    MoveCall(SuiProgrammableMoveCall),
    TransferObjects,
    SplitCoins,
    MergeCoins,
    Publish,
    MakeMoveVec,
}

pub struct SuiProgrammableMoveCall {
    pub package: ObjectID,
    pub module: String,
    pub function: String,
    pub type_arguments: Vec<TypeTag>,
    pub arguments: Vec<SuiArgument>,
}

pub enum SuiArgument {
    GasCoin,
    Input(u16),
    Result(u16),
    NestedResult(u16, u16),
}

pub struct SuiGasData {
    pub payment: Vec<SuiObjectRef>,
    pub owner: SuiAddress,
    pub price: u64,
    pub budget: u64,
}

pub struct SuiChangeEpoch {
    pub epoch: u64,
    pub storage_charge: u64,
    pub computation_charge: u64,
    pub storage_rebate: u64,
    pub epoch_start_timestamp_ms: u64,
}

pub struct SuiGenesisTransaction {
    pub objects: Vec<ObjectID>,
}

pub struct SuiConsensusCommitPrologue {
    pub epoch: u64,
    pub round: u64,
    pub commit_timestamp_ms: u64,
}

pub struct SuiConsensusCommitPrologueV2 {
    pub epoch: u64,
    pub round: u64,
    pub commit_timestamp_ms: u64,
    pub consensus_commit_digest: TransactionDigest,
}

pub struct SuiConsensusCommitPrologueV3 {
    pub epoch: u64,
    pub round: u64,
    pub sub_dag_index: Option<u64>,
    pub commit_timestamp_ms: u64,
    pub consensus_commit_digest: TransactionDigest,
}

pub struct SuiConsensusCommitPrologueV4 {
    pub epoch: u64,
    pub round: u64,
    pub sub_dag_index: Option<u64>,
    pub commit_timestamp_ms: u64,
    pub consensus_commit_digest: TransactionDigest,
    pub additional_state_digest: TransactionDigest,
}

pub struct SuiAuthenticatorStateUpdate;
pub struct SuiRandomnessStateUpdate;
pub struct SuiEndOfEpochTransaction;

// --- Common types ------------------------------------------------------------

pub struct OwnedObjectRef {
    pub reference: SuiObjectRef,
    pub owner: Owner,
}

pub struct SuiObjectRef {
    pub object_id: ObjectID,
    pub version: SequenceNumber,
    pub digest: ObjectDigest,
}

pub enum SuiExecutionStatus {
    Success,
    Failure { error: String },
}

pub struct SuiAccumulatorEvent {
    pub accumulator_obj: ObjectID,
    pub address: SuiAddress,
    pub ty: TypeTag,
    pub operation: SuiAccumulatorOperation,
    pub value: SuiAccumulatorValue,
}

pub enum SuiAccumulatorOperation {
    Merge,
    Split,
}

pub enum SuiAccumulatorValue {
    Integer(u64),
    IntegerTuple(u64, u64),
    EventDigest(u64, ObjectDigest),
}

pub struct SuiMoveAbort {
    pub location: String,
    pub abort_code: u64,
}

pub struct BalanceChange {
    pub owner: Owner,
    pub coin_type: TypeTag,
    pub amount: i128,
}

pub enum ObjectChange {
    Published {
        package_id: ObjectID,
        version: SequenceNumber,
        digest: ObjectDigest,
        modules: Vec<String>,
    },
    Transferred {
        sender: SuiAddress,
        recipient: Owner,
        object_type: StructTag,
        object_id: ObjectID,
        version: SequenceNumber,
        digest: ObjectDigest,
    },
    Mutated {
        sender: SuiAddress,
        owner: Owner,
        object_type: StructTag,
        object_id: ObjectID,
        version: SequenceNumber,
        previous_version: SequenceNumber,
        digest: ObjectDigest,
    },
    Deleted {
        sender: SuiAddress,
        object_type: StructTag,
        object_id: ObjectID,
        version: SequenceNumber,
    },
    Wrapped {
        sender: SuiAddress,
        object_type: StructTag,
        object_id: ObjectID,
        version: SequenceNumber,
    },
    Created {
        sender: SuiAddress,
        owner: Owner,
        object_type: StructTag,
        object_id: ObjectID,
        version: SequenceNumber,
        digest: ObjectDigest,
    },
}

/**
 * hREA recipe_flow zome internal data structures
 *
 * Required by the zome itself, and for any DNA-local zomes interacting with its
 * storage API directly.
 *
 * @package hREA
 */
use hdk::prelude::*;

use hc_zome_dna_auth_resolver_core::AvailableCapability;
use hdk_records::{
    RecordAPIResult, DataIntegrityError,
    MaybeUndefined,
    record_interface::Updateable,
    generate_record_entry,
};
use vf_measurement::QuantityValue;

pub use vf_attributes_hdk::{
    ActionId,
    DateTime, FixedOffset,
    ExternalURL,
    RecipeFlowAddress,
    LocationAddress,
    AgentAddress,
    EconomicResourceAddress,
    ProcessAddress,
    ResourceSpecificationAddress,
    ProcessSpecificationAddress,
    RecipeProcessAddress,
};

use vf_actions::{ validate_flow_action };

use hc_zome_rea_recipe_flow_rpc::{ CreateRequest, UpdateRequest };

// :SHONK: needed as re-export in zome logic to allow validation logic to parse entries
pub use hdk_records::record_interface::Identified;

//--------------- ZOME CONFIGURATION ATTRIBUTES ----------------

// :TODO: remove this, replace with reference to appropriate namespacing of zome config
#[derive(Clone, Serialize, Deserialize, SerializedBytes, PartialEq, Debug)]
pub struct DnaConfigSlice {
    pub recipe_flow: RecipeFlowZomeConfig,
}

#[derive(Clone, Serialize, Deserialize, SerializedBytes, PartialEq, Debug)]
pub struct RecipeFlowZomeConfig {
    pub index_zome: String,
    pub process_index_zome: Option<String>,
    pub agent_index_zome: Option<String>,
}

//---------------- RECORD INTERNALS & VALIDATION ----------------

#[derive(Serialize, Deserialize, Debug, SerializedBytes, Clone)]
pub struct EntryData {
    pub resource_quantity: Option<QuantityValue>,
    pub effort_quantity: Option<QuantityValue>,
    pub action: ActionId,
    pub note: Option<String>,
    pub state: Option<String>,
    pub resource_conforms_to: Option<ResourceSpecificationAddress>,
    pub stage: Option<ProcessSpecificationAddress>,
    pub recipe_input_of: Option<RecipeProcessAddress>,
    pub recipe_output_of: Option<RecipeProcessAddress>,
    pub _nonce: Bytes,
}

impl EntryData {
    pub fn validate_recipe_flow(&self) -> Result<(), String> {
        Ok(())
    }
}

generate_record_entry!(EntryData, RecipeFlowAddress, EntryStorage);

//---------------- Holochain App Entry And Link Types Setup ----------------

#[hdk_entry_defs(skip_hdk_extern = true)]
#[unit_enum(EntryTypesUnit)]
pub enum EntryTypes {
    RecipeFlow(EntryStorage),
    #[entry_def(visibility = "private")]
    AvailableCapability(AvailableCapability)
}

impl From<EntryStorage> for EntryTypes
{
    fn from(e: EntryStorage) -> EntryTypes
    {
        EntryTypes::RecipeFlow(e)
    }
}
impl TryFrom<AvailableCapability> for EntryTypes {
    type Error = WasmError;

    fn try_from(e: AvailableCapability) -> Result<EntryTypes, Self::Error>
    {
        Ok(EntryTypes::AvailableCapability(e))
    }
}

#[hdk_link_types(skip_no_mangle = true)]
pub enum LinkTypes {
    // relates to dna-auth-resolver mixin
    // and remote authorizations
    AvailableCapability
}


//---------------- CREATE ----------------

/// Pick relevant fields out of I/O record into underlying DHT entry
impl TryFrom<CreateRequest> for EntryData {
    type Error = DataIntegrityError;

    fn try_from(e: CreateRequest) -> RecordAPIResult<EntryData> {
        Ok(EntryData {
            note: e.note.to_owned().into(),
            state: e.note.to_owned().into(),
            action: e.action.to_owned().into(),
            resource_quantity: e.resource_quantity.to_owned().into(),
            effort_quantity: e.effort_quantity.to_owned().into(),
            resource_conforms_to: e.resource_conforms_to.to_owned().into(),
            // recipe_clause_of: e.recipe_clause_of.to_owned().into(),
            stage: e.stage.to_owned().into(),
            recipe_input_of: e.recipe_input_of.to_owned().into(),
            recipe_output_of: e.recipe_output_of.to_owned().into(),
            _nonce: random_bytes(32)?,
        })
    }
}

//---------------- UPDATE ----------------

/// Handles update operations by merging any newly provided fields
impl Updateable<UpdateRequest> for EntryData {
    fn update_with(&self, e: UpdateRequest) -> RecordAPIResult<EntryData> {
        Ok(EntryData {
            note: if e.note== MaybeUndefined::Undefined { self.note.to_owned() } else { e.note.to_owned().into() },
            state: if e.note== MaybeUndefined::Undefined { self.note.to_owned() } else { e.note.to_owned().into() },
            action: if !e.action.is_some() { self.action.to_owned() } else { e.action.to_owned().unwrap() },
            resource_quantity: if e.resource_quantity== MaybeUndefined::Undefined { self.resource_quantity.to_owned() } else { e.resource_quantity.to_owned().into() },
            effort_quantity: if e.effort_quantity== MaybeUndefined::Undefined { self.effort_quantity.to_owned() } else { e.effort_quantity.to_owned().into() },
            resource_conforms_to: if e.resource_conforms_to == MaybeUndefined::Undefined { self.resource_conforms_to.to_owned() } else { e.resource_conforms_to.to_owned().into() },
            // recipe_clause_of: if e.recipe_clause_of == MaybeUndefined::Undefined { self.recipe_clause_of.to_owned() } else { e.recipe_clause_of.to_owned().into() },
            stage: if e.stage == MaybeUndefined::Undefined { self.stage.to_owned() } else { e.stage.to_owned().into() },
            recipe_input_of: if e.recipe_input_of == MaybeUndefined::Undefined { self.recipe_input_of.to_owned() } else { e.recipe_input_of.to_owned().into() },
            recipe_output_of: if e.recipe_output_of == MaybeUndefined::Undefined { self.recipe_output_of.to_owned() } else { e.recipe_output_of.to_owned().into() },
            _nonce: self._nonce.to_owned(),
        })
    }
}

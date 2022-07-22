/**
 * Holo-REA agent zome internal data structures
 *
 * Required by the zome itself, and for any DNA-local zomes interacting with its
 * storage API directly.
 *
 * @package Holo-REA
 */
use hdk::prelude::*;

use hdk_records::{
    RecordAPIResult, DataIntegrityError,
    generate_record_entry,
    record_interface::{Updateable},
};

use vf_attributes_hdk::{
    ExternalURL,
};

use hc_zome_rea_agent_rpc::{ CreateRequest, UpdateRequest };

pub use vf_attributes_hdk::AgentAddress;
pub use hc_zome_rea_agent_storage_consts::AGENT_ENTRY_TYPE;

// :SHONK: needed as re-export in zome logic to allow validation logic to parse entries
pub use hdk_records::record_interface::Identified;

//--------------- ZOME CONFIGURATION ATTRIBUTES ----------------

// :TODO: remove this, replace with reference to appropriate namespacing of zome config
#[derive(Clone, Serialize, Deserialize, SerializedBytes, PartialEq, Debug)]
pub struct DnaConfigSlice {
    pub agent: AgentZomeConfig,
}

#[derive(Clone, Serialize, Deserialize, SerializedBytes, PartialEq, Debug)]
pub struct AgentZomeConfig {
    pub index_zome: String,
}

//---------------- RECORD INTERNALS & VALIDATION ----------------

#[derive(Clone, Serialize, Deserialize, SerializedBytes, Debug)]
pub struct EntryData {
    pub name: String,
    pub agent_type: String,
    pub image: Option<ExternalURL>,
    pub classified_as: Option<Vec<ExternalURL>>,
    pub note: Option<String>,
    pub _nonce: Bytes,
}

generate_record_entry!(EntryData, AgentAddress, EntryStorage);

//---------------- CREATE ----------------

/// Pick relevant fields out of I/O record into underlying DHT entry
impl TryFrom<CreateRequest> for EntryData {
    type Error = DataIntegrityError;

    fn try_from(e: CreateRequest) -> RecordAPIResult<EntryData> {
        Ok(EntryData {
            name: e.name.into(),
            agent_type: e.agent_type.into(),
            image: e.image.into(),
            classified_as: e.classified_as.into(),
            note: e.note.into(),
            _nonce: random_bytes(32)?,
        })
    }
}

//---------------- UPDATE ----------------

/// Handles update operations by merging any newly provided fields
impl Updateable<UpdateRequest> for EntryData {
    fn update_with(&self, e: UpdateRequest) -> EntryData {
        EntryData {
            name: if !e.name.is_some() { self.name.to_owned() } else { e.name.to_owned().unwrap() },
            agent_type: self.agent_type.to_owned(), //assume cannot update agent_type
            image: if !e.image.is_some() { self.image.to_owned() } else { e.image.to_owned().into() },
            classified_as: if !e.classified_as.is_some() { self.classified_as.to_owned() } else { e.classified_as.to_owned().into() },
            note: if !e.note.is_some() { self.note.to_owned() } else { e.note.to_owned().into() },
            _nonce: self._nonce.to_owned(),
        }
    }
}

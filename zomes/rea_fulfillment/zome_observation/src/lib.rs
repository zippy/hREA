/**
 * Holo-REA fulfillment remote index zome API definition
 *
 * Manages indexes for querying `EconomicEvents` against remote `Fulfillments`.
 *
 * Defines the top-level zome configuration needed by Holochain's build system
 * to bundle the app. This basically involves wiring up the helper methods from the
 * related `_lib` module into a packaged zome WASM binary.
 *
 * @package Holo-REA
 */
use hdk::prelude::*;

use hc_zome_rea_fulfillment_lib_destination::*;
use hc_zome_rea_fulfillment_rpc::*;
use hc_zome_rea_fulfillment_storage_consts::*;
use hc_zome_rea_economic_event_storage_consts::EVENT_ENTRY_TYPE;

#[hdk_extern]
fn entry_defs(_: ()) -> ExternResult<EntryDefsCallbackResult> {
    Ok(EntryDefsCallbackResult::from(vec![
        Path::entry_def(),
        EntryDef {
            id: FULFILLMENT_ENTRY_TYPE.into(),
            visibility: EntryVisibility::Public,
            crdt_type: CrdtType,
            required_validations: 1.into(),
            required_validation_type: RequiredValidationType::default(),
        }
    ]))
}

#[hdk_extern]
fn fulfillment_created(CreateParams { fulfillment }: CreateParams) -> ExternResult<ResponseData> {
    Ok(receive_create_fulfillment(FULFILLMENT_ENTRY_TYPE, EVENT_ENTRY_TYPE, fulfillment)?)
}

#[hdk_extern]
fn get_fulfillment(ByAddress { address }: ByAddress<FulfillmentAddress>) -> ExternResult<ResponseData> {
    Ok(receive_get_fulfillment(FULFILLMENT_ENTRY_TYPE, address)?)
}

#[hdk_extern]
fn fulfillment_updated(UpdateParams { fulfillment }: UpdateParams) -> ExternResult<ResponseData> {
    Ok(receive_update_fulfillment(FULFILLMENT_ENTRY_TYPE, EVENT_ENTRY_TYPE, fulfillment)?)
}

#[hdk_extern]
fn fulfillment_deleted(ByHeader { address }: ByHeader) -> ExternResult<bool> {
    Ok(receive_delete_fulfillment(address)?)
}

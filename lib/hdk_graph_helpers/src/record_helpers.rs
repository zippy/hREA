/**
 * Record-handling abstractions for Holochain apps
 *
 * Allows for data layers which behave like a traditional graph database, where
 * 'records' are the core abstraction, managed by complex arrangements of DHT
 * entries and links.
 *
 * @package HoloREA
 * @since   2019-07-02
 */

use std::convert::TryFrom;
use hdk::{
    holochain_persistence_api::cas::content::Address,
    holochain_core_types::{
        entry::{
            Entry::App as AppEntry,
            entry_type::AppEntryType,
            AppEntryValue,
        },
    },
    error::{ ZomeApiResult, ZomeApiError },
    entry_address,
    commit_entry,
    update_entry,
    remove_entry,
    link_entries,
    utils:: {
        get_as_type,    // :TODO: switch this method to one which doesn't consume the input
    },
};

use super::{
    identifiers::RECORD_INITIAL_ENTRY_LINK_TAG,
    record_interface::Updateable,
};



// CREATE

/// Creates a new record in the DHT, assigns it a predictable `base` (static) id, and returns a tuple of
/// the `base` id and initial record `entry` data.
/// It is recommended that you include a creation timestamp in newly created records, to avoid
/// them conflicting with previously entered entries that may be of the same content.
pub fn create_record<E, C, A, S>(
    base_entry_type: S,
    entry_type: S,
    initial_entry_link_type: &str,
    create_payload: C,
) -> ZomeApiResult<(A, E)>
    where E: Clone + Into<AppEntryValue>,
        C: Into<E>,
        S: Into<AppEntryType>,
        A: From<Address>,
{
    // convert the type's CREATE payload into internal struct via built-in conversion trait
    let entry_struct: E = create_payload.into();
    // clone entry for returning to caller
    // :TODO: should not need to do this if AppEntry stops consuming the value
    let entry_resp = entry_struct.clone();

    // write underlying entry and get initial address (which will become record ID)
    let entry = AppEntry(entry_type.into(), entry_struct.into());
    let address = commit_entry(&entry)?;

    // create a base entry pointer
    let base_address = create_base_entry(&(base_entry_type.into()), &address)?;
    // :NOTE: link is just for inference by external tools, it's not actually needed to query
    link_entries(&base_address, &address, initial_entry_link_type, RECORD_INITIAL_ENTRY_LINK_TAG)?;

    Ok((A::from(base_address), entry_resp))
}

/// Creates a `base` entry address- an entry consisting only of a pointer to some other referenced
/// `entry`. The address of the `base` entry (the alias the changing `entry` will be identified by
/// within this network) is returned.
pub fn create_base_entry(
    base_entry_type: &AppEntryType,
    referenced_address: &Address,
) -> ZomeApiResult<Address> {
    let base_entry = AppEntry(base_entry_type.clone().into(), referenced_address.into());
    commit_entry(&base_entry)
}




// READ

/// Read a record's entry data by its `base` (static) id.
pub fn read_record_entry<T: TryFrom<AppEntryValue>, A: AsRef<Address>>(
    address: &A,
) -> ZomeApiResult<T> {
    // read base entry to determine dereferenced entry address
    let data_address = get_dereferenced_address(address.as_ref());

    // return retrieval error or attempt underlying type fetch
    match data_address {
        Ok(addr) => get_as_type(addr),
        Err(e) => Err(e),
    }
}

/// Query the `entry` address for a given `base` address.
pub fn get_dereferenced_address(base_address: &Address) -> ZomeApiResult<Address> {
    get_as_type(base_address.clone())
}





// UPDATE

/// Updates a record in the DHT by its `base` (static) id.
/// The way in which the input update payload is applied to the existing
/// entry data is up to the implementor of `Updateable<U>` for the entry type.
pub fn update_record<E, U, A, S>(
    entry_type: S,
    address: &A,
    update_payload: &U,
) -> ZomeApiResult<E>
    where E: Clone + TryFrom<AppEntryValue> + Into<AppEntryValue> + Updateable<U>,
        S: Into<AppEntryType>,
        A: AsRef<Address>,
{
    // read base entry to determine dereferenced entry address
    let data_address = get_dereferenced_address(address.as_ref())?;
    let prev_entry: E = get_as_type(data_address.clone())?;

    // perform update logic
    let new_entry = prev_entry.update_with(update_payload);

    // clone entry for returning to caller
    // :TODO: should not need to do this if AppEntry stops consuming the value
    let entry_resp = new_entry.clone();

    // store updated entry back to the DHT if there was a change
    let entry = AppEntry(entry_type.into(), new_entry.into());

    // :IMPORTANT: only write if data has changed, otherwise core gets in infinite loops
    // @see https://github.com/holochain/holochain-rust/issues/1662
    let new_hash = entry_address(&entry)?;
    if new_hash != data_address {
        update_entry(entry, &data_address)?;
    }

    Ok(entry_resp)
}




// DELETE

/// Removes a record of the given `base` from the DHT by marking it as deleted.
/// Links are not affected so as to retain a link to the referencing information, which may now need to be updated.
pub fn delete_record<T>(address: &dyn AsRef<Address>) -> ZomeApiResult<bool>
    where T: TryFrom<AppEntryValue>
{
    // read base entry to determine dereferenced entry address
    let data_address = get_dereferenced_address(address.as_ref());

    match data_address {
        // note that we're relying on the deletions to be paired in using this as an existence check
        Ok(addr) => {
            let entry_data: ZomeApiResult<T> = get_as_type(addr.clone());
            match entry_data {
                Ok(_) => {
                    remove_entry(address.as_ref())?;
                    remove_entry(&addr)?;
                    Ok(true)
                },
                Err(_) => Err(ZomeApiError::ValidationFailed("incorrect record type specified for deletion".to_string())),
            }
        },
        Err(_) => Ok(false),
    }
}

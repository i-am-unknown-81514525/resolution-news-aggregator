#[cfg(target_arch = "wasm32")]
use idb::{Factory};
use idb::{DatabaseEvent, KeyPath, ObjectStoreParams, Request, TransactionMode};
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys::Uint8Array;

use rkyv::{to_bytes, rancor::Error};
use crate::local_unify::LocalUnify;

#[cfg(target_arch = "wasm32")]
fn store_unify(factory: &mut Factory, item: &LocalUnify) -> () {
    let mut factory = factory.open("unify", Some(1)).unwrap();
    let decoded = to_bytes::<Error>(item).unwrap().into_vec();
    let array = Uint8Array::new_with_length(decoded.len() as u32);
    array.copy_from(&decoded);
    let idx = item.idx;
    factory.on_upgrade_needed(|resp| {
        let database = resp.database().unwrap();
        let store_params = ObjectStoreParams::new();
        database
            .create_object_store("unify", store_params)
            .unwrap();
    });
    factory.on_success(move |resp| {
        let database = resp.database().unwrap();
        let transaction = database.transaction(&["unify"], TransactionMode::ReadWrite).unwrap();
        let store = transaction.object_store("unify").unwrap();
        store.put(&array, Some(&idx.into())).unwrap();
        transaction.commit().unwrap();
    });
}
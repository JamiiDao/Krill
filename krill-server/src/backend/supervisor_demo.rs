use dioxus::prelude::*;
use krill_common::{Blake3BytesRedacted, OrganizationInfo, StorageKeys};

use crate::backend::KRILL_STORAGE;

#[post("/create-organization")]
pub async fn create_organization(
    info: Vec<u8>,
) -> ServerFnResult<dioxus::fullstack::axum_core::response::Response> {
    use krill_common::OrganizationInfo;

    let mut org_details =
        bitcode::decode::<OrganizationInfo>(&info).or(Err(ServerFnError::ServerError {
            message: "Invalid payload".to_string(),
            code: 402,
            details: None,
        }))?;
    org_details.name = org_details.name.trim().to_string();

    tracing::info!("Create org details: {:?}", &org_details);

    let setup_key = Blake3BytesRedacted::new();

    let store = KRILL_STORAGE.get().unwrap();
    let store_keyspace = store.organizations_keyspace();
    let keys_keyspace = store.organization_setup_key_keyspace();

    let key = StorageKeys::gen_store_key_owned(org_details.name.as_bytes());

    if let Err(error) = store
        .set_bytes_op(store_keyspace, key.clone(), org_details)
        .await
    {
        return Ok(crate::server_outcome_response::<Blake3BytesRedacted>(
            Result::Err(error),
        ));
    }

    if let Err(error) = store
        .set_bytes_op(keys_keyspace, key, setup_key.clone())
        .await
    {
        return Ok(crate::server_outcome_response::<Blake3BytesRedacted>(
            Result::Err(error),
        ));
    }

    Ok(crate::server_outcome_response(Result::Ok(setup_key)))
}

#[get("/get-organization/:target")]
pub async fn get_organization(
    target: String,
) -> ServerFnResult<dioxus::fullstack::axum_core::response::Response> {
    let store = KRILL_STORAGE.get().unwrap();
    let keyspace = store.organizations_keyspace();

    let key = StorageKeys::gen_store_key_owned(target.as_bytes());

    let outcome = store.get_bytes_op(keyspace, key).await;

    Ok(crate::server_outcome_response::<Option<Vec<u8>>>(outcome))
}

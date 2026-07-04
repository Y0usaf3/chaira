use leptos::logging::log;
use leptos::prelude::*;
use models::{BaseId, Field, FieldConfig, FieldId, Record, RecordId, TableId, Value};
use std::collections::HashMap;
use std::time::Instant;

// NOTE: better use Json enc/dec when using wrapper types or complex types
use leptos::server_fn::codec::Json;

#[server]
pub async fn get_table_data(
    base_id: BaseId,
    table_id: TableId,
) -> Result<(Vec<Field>, Vec<Record>), ServerFnError> {
    let start = Instant::now();
    log!("get_table_data: base={base_id:?}, table={table_id:?}");
    let service = crate::get_authenticated_service().await?;
    let uid = service.id().clone();
    let mut ts = service::table::TableService::new(table_id, base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;
    let result = ts
        .get_full_data(None)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get table data: {e:?}")));
    if let Ok((fields, records)) = &result {
        log!(
            "get_table_data: {} fields, {} records (took {}ms)",
            fields.len(),
            records.len(),
            start.elapsed().as_millis()
        );
    } else {
        log!("get_table_data: failed after {}ms", start.elapsed().as_millis());
    }
    result
}

#[server(input = Json, output = Json)]
pub async fn create_table_field(
    base_id: BaseId,
    table_id: TableId,
    name: String,
    config: FieldConfig,
) -> Result<Field, ServerFnError> {
    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let uid = service.id().clone();
    let mut ts = service::table::TableService::new(table_id.clone(), base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;
    let insert = models::InsertField {
        name: name.clone(),
        description: None,
        is_primary: false,
        is_nullable: true,
        is_unique: false,
        order: 0,
        config,
    };
    let result = ts
        .create_field(insert)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create field: {e:?}")));
    
    if result.is_ok() {
        log!("create_table_field: '{}' created successfully (took {}ms)", name, start.elapsed().as_millis());
    } else {
        log!("create_table_field: failed after {}ms", start.elapsed().as_millis());
    }
    result
}

#[server]
pub async fn update_cell_value(
    base_id: BaseId,
    table_id: TableId,
    record_id: RecordId,
    field_id: FieldId,
    value: Value,
) -> Result<Record, ServerFnError> {
    let start = Instant::now();
    log!(
        "update_cell_value: base={base_id:?}, table={table_id:?}, record={record_id:?}, field={field_id:?}, value={value:?}"
    );
    let service = crate::get_authenticated_service().await?;
    let uid = service.id().clone();
    let mut ts = service::table::TableService::new(table_id.clone(), base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;

    let patch = models::RecordPatch::new(Some(vec![(field_id.id_str(), value)]));
    let result = ts
        .update_record(record_id, patch)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update record: {e:?}")));
    if let Ok(ref rec) = result {
        log!("update_cell_value: updated record {:?} (took {}ms)", rec.id, start.elapsed().as_millis());
    } else {
        log!("update_cell_value: failed after {}ms", start.elapsed().as_millis());
    }
    result
}

#[server]
pub async fn create_table_record(
    base_id: BaseId,
    table_id: TableId,
) -> Result<Record, ServerFnError> {
    let start = Instant::now();
    log!("create_table_record: base={base_id:?}, table={table_id:?}");
    let service = crate::get_authenticated_service().await?;
    let uid = service.id().clone();
    let mut ts = service::table::TableService::new(table_id.clone(), base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;
    let insert = models::InsertRecord::new(table_id, HashMap::new());
    let result = ts
        .create_record(insert)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create record: {e:?}")));
    if let Ok(ref rec) = result {
        log!("create_table_record: created record {:?} (took {}ms)", rec.id, start.elapsed().as_millis());
    } else {
        log!("create_table_record: failed after {}ms", start.elapsed().as_millis());
    }
    result
}


use leptos::logging::log;
use leptos::prelude::*;
use models::{BaseId, Field, FieldConfig, FieldId, Record, RecordId, TableId, Value};

// NOTE: better use Json enc/dec when using wrapper types or complex types
use leptos::server_fn::codec::Json;

#[server]
pub async fn get_table_data(
    base_id: BaseId,
    table_id: TableId,
) -> Result<(Vec<Field>, Vec<Record>), ServerFnError> {
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
            "get_table_data: {} fields, {} records",
            fields.len(),
            records.len()
        );
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
    let service = crate::get_authenticated_service().await?;
    let uid = service.id().clone();
    let mut ts = service::table::TableService::new(table_id.clone(), base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;
    let insert = models::InsertField {
        name,
        description: None,
        is_primary: false,
        is_nullable: true,
        is_unique: false,
        order: 0,
        config,
    };
    ts.create_field(insert)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create field: {e:?}")))
}


#[server]
pub async fn update_cell_value(
    base_id: BaseId,
    table_id: TableId,
    record_id: RecordId,
    field_id: FieldId,
    value: Value,
) -> Result<Record, ServerFnError> {
    log!(
        "update_cell_value: base={base_id:?}, table={table_id:?}, record={record_id:?}, field={field_id:?}, value={value:?}"
    );
    let service = crate::get_authenticated_service().await?;
    let uid = service.id().clone();
    let mut ts = service::table::TableService::new(table_id.clone(), base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;
    let field_cfg = ts
        .get_field_config(field_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get field config: {e:?}")))?;

    let patch = models::RecordPatch::new(Some(vec![(field_cfg.name, value)]));
    let result = ts
        .update_record(record_id, patch)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update record: {e:?}")));
    if let Ok(ref rec) = result {
        log!("update_cell_value: updated record {:?}", rec.id);
    }
    result
}


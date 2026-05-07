use leptos::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct BaseTable {
    pub name: String,
    pub id: String,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct TableField {
    pub id: String,
    pub name: String,
    pub config: charac::models::field::FieldConfig,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, PartialEq)]
pub struct TableRecord {
    pub id: String,
    pub cells: std::collections::HashMap<String, String>, // field_name -> value_string
}

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct TableData {
    pub fields: Vec<TableField>,
    pub records: Vec<TableRecord>,
}

#[server]
pub async fn get_base_tables(base_id: String) -> Result<Vec<BaseTable>, ServerFnError> {
    use charac::models::ids::BaseId;
    use std::time::Instant;
    use surrealdb::types::RecordId;
    use surrealdb::types::ToSql;

    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let mut user_service = service;

    let base_record_id = if base_id.contains(':') {
        RecordId::parse_simple(&base_id)
    } else {
        RecordId::parse_simple(format!("base:{}", base_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the base id"))?;

    let base_id_typed = BaseId(base_record_id);
    user_service
        .open_base(base_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("Opening Base failed: {e:?}")))?;

    let tables = user_service
        .current_base
        .as_ref()
        .unwrap()
        .list_tables()
        .await
        .map_err(|e| ServerFnError::new(format!("Listing Tables failed: {e:?}")))?;

    let base_tables = tables
        .into_iter()
        .map(|t| BaseTable {
            name: t.name,
            id: t.id.unwrap().0.to_sql(),
        })
        .collect();

    let duration = start.elapsed().as_millis();
    println!("[get_base_tables] finished in {}ms", duration);

    Ok(base_tables)
}

#[server]
pub async fn create_table_in_base(
    base_id: String,
    name: String,
) -> Result<BaseTable, ServerFnError> {
    use charac::models::ids::BaseId;
    use std::time::Instant;
    use surrealdb::types::RecordId;
    use surrealdb::types::ToSql;

    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let mut user_service = service;

    let base_record_id = if base_id.contains(':') {
        RecordId::parse_simple(&base_id)
    } else {
        RecordId::parse_simple(format!("base:{}", base_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the base id"))?;

    let base_id_typed = BaseId(base_record_id);
    user_service
        .open_base(base_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;

    let table = user_service
        .current_base
        .as_ref()
        .unwrap()
        .create_table(name)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;

    let duration = start.elapsed().as_millis();
    println!("[create_table_in_base] finished in {}ms", duration);

    Ok(BaseTable {
        name: table.name,
        id: table.id.unwrap().0.to_sql(),
    })
}

#[server]
pub async fn get_table_data(base_id: String, table_id: String) -> Result<TableData, ServerFnError> {
    use charac::models::ids::{BaseId, TableId};
    use std::time::Instant;
    use surrealdb::types::RecordId;
    use surrealdb::types::ToSql;

    let start = Instant::now();

    let mut user_service = crate::get_authenticated_service().await?;

    let base_record_id = if base_id.contains(':') {
        RecordId::parse_simple(&base_id)
    } else {
        RecordId::parse_simple(format!("base:{}", base_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the base id"))?;
    let base_id_typed = BaseId(base_record_id);

    user_service
        .open_base(base_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("Opening Base failed: {e:?}")))?;

    let table_record_id = RecordId::parse_simple(table_id.as_str())
        .ok()
        .ok_or(ServerFnError::new("coudlnt parse the table id"))?;
    let table_id_typed = TableId(table_record_id);

    let table_service = user_service
        .current_base
        .as_ref()
        .unwrap()
        .open_table(table_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let (fields, records) = table_service
        .get_full_data(None)
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let table_fields = fields
        .into_iter()
        .map(|f| TableField {
            id: f.id.clone().unwrap().0.to_sql(),
            name: f.name,
            config: f.config,
        })
        .collect::<Vec<_>>();

    let table_records = records
        .into_iter()
        .map(|r| {
            let mut cells = std::collections::HashMap::new();
            for (field_name, cell_value) in r.cells {
                cells.insert(field_name, cell_value.value.to_string());
            }
            TableRecord {
                id: r.id.clone().unwrap().0.to_sql(),
                cells,
            }
        })
        .collect();

    let duration = start.elapsed().as_millis();
    println!("[get_table_data] finished in {}ms", duration);

    Ok(TableData {
        fields: table_fields,
        records: table_records,
    })
}

#[server]
pub async fn delete_record(
    base_id: String,
    table_id: String,
    record_id: String,
) -> Result<(), ServerFnError> {
    use charac::models::ids::{BaseId, RecordId as CharacRecordId, TableId};
    use std::time::Instant;
    use surrealdb::types::RecordId;

    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let mut user_service = service;

    let base_record_id = if base_id.contains(':') {
        RecordId::parse_simple(&base_id)
    } else {
        RecordId::parse_simple(format!("base:{}", base_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the base id"))?;
    let base_id_typed = BaseId(base_record_id);

    user_service
        .open_base(base_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;

    let table_record_id = if table_id.contains(':') {
        RecordId::parse_simple(&table_id)
    } else {
        RecordId::parse_simple(format!("table:{}", table_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the table id"))?;

    let table_id_typed = TableId(table_record_id);

    let table_service = user_service
        .current_base
        .as_ref()
        .unwrap()
        .open_table(table_id_typed.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let record_record_id = RecordId::parse_simple(&record_id)
        .map_err(|_| ServerFnError::new("coudlnt parse the record id"))?;

    table_service
        .delete_record(CharacRecordId(record_record_id))
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let duration = start.elapsed().as_millis();
    println!("[delete_record] finished in {}ms", duration);

    Ok(())
}

#[server]
pub async fn create_record(
    base_id: String,
    table_id: String,
) -> Result<TableRecord, ServerFnError> {
    use charac::models::ids::{BaseId, TableId};
    use charac::models::record::InsertRecord;
    use std::time::Instant;
    use surrealdb::types::{RecordId, ToSql};

    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let mut user_service = service;

    let base_record_id = if base_id.contains(':') {
        RecordId::parse_simple(&base_id)
    } else {
        RecordId::parse_simple(format!("base:{}", base_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the base id"))?;
    let base_id_typed = BaseId(base_record_id);

    user_service
        .open_base(base_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;

    let table_record_id = if table_id.contains(':') {
        RecordId::parse_simple(&table_id)
    } else {
        RecordId::parse_simple(format!("table:{}", table_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the table id"))?;

    let table_id_typed = TableId(table_record_id);

    let table_service = user_service
        .current_base
        .as_ref()
        .unwrap()
        .open_table(table_id_typed.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let record = table_service
        .create_record(InsertRecord::new(
            table_id_typed,
            std::collections::HashMap::new(),
        ))
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let duration = start.elapsed().as_millis();
    println!("[create_record] finished in {}ms", duration);

    Ok(TableRecord {
        id: record.id.unwrap().0.to_sql(),
        cells: std::collections::HashMap::new(),
    })
}

#[server]
pub async fn update_record_cell(
    base_id: String,
    table_id: String,
    record_id: String,
    field_id: String,
    new_value: String,
) -> Result<(), ServerFnError> {
    use charac::db::DB;
    use charac::models::field::{Field, FieldConfig, NumberConfig, TextConfig};
    use charac::models::ids::{BaseId, RecordId as CharacRecordId, TableId};
    use charac::models::record::RecordPatch;
    use charac::models::record::cell::{
        CellValue, Email, LongTextValue, NumberValue, PhoneValue, SingleLineValue, UrlValue, Value,
    };
    use std::time::Instant;
    use surrealdb::types::RecordId;

    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let mut user_service = service;

    let base_record_id = if base_id.contains(':') {
        RecordId::parse_simple(&base_id)
    } else {
        RecordId::parse_simple(format!("base:{}", base_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the base id"))?;
    let base_id_typed = BaseId(base_record_id);

    user_service
        .open_base(base_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("Opening Base failed: {e:?}")))?;

    let table_record_id = if table_id.contains(':') {
        RecordId::parse_simple(&table_id)
    } else {
        RecordId::parse_simple(format!("table:{}", table_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the table id"))?;

    let table_id_typed = TableId(table_record_id);

    let table_service = user_service
        .current_base
        .as_ref()
        .unwrap()
        .open_table(table_id_typed.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    // Fetch field config for validation
    let field_record_id = RecordId::parse_simple(&field_id)
        .map_err(|_| ServerFnError::new("Couldn't parse the field id"))?;

    let mut res = DB
        .query("SELECT * FROM $field WHERE table = $table AND is_deleted = false")
        .bind(("field", field_record_id))
        .bind(("table", table_id_typed))
        .await?;
    let field: Field = res
        .take::<Option<Field>>(0)?
        .ok_or(ServerFnError::new("Field not found"))?;

    // Parse empty string as None for optional values
    let value_opt = if new_value.trim().is_empty() {
        None
    } else {
        Some(new_value.clone())
    };

    // Validate and convert value based on field config
    let value = match &field.config {
        FieldConfig::Text(text_config) => match text_config {
            TextConfig::SingleLine { default, .. } => {
                let v = SingleLineValue::new(default.clone(), value_opt)
                    .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
                Value::SingleLine(v)
            }
            TextConfig::LongText { rich_text } => {
                let v = LongTextValue::new(new_value, *rich_text)
                    .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
                Value::LongText(Box::new(v))
            }
            TextConfig::Email => {
                let v = Email::new(new_value)
                    .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
                Value::Email(v)
            }
            TextConfig::URL => {
                let v = UrlValue::new(new_value)
                    .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
                Value::URL(v)
            }
            TextConfig::Phone => {
                let v = PhoneValue::new(new_value, None)
                    .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
                Value::Phone(v)
            }
        },
        FieldConfig::Number(num_config) => {
            match num_config {
                NumberConfig::Number { default } => {
                    let num_opt = if new_value.trim().is_empty() {
                        None
                    } else {
                        Some(
                            new_value
                                .parse::<usize>()
                                .map_err(|_| ServerFnError::new("Invalid number format"))?,
                        )
                    };
                    let v = NumberValue::new(num_opt, *default)
                        .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
                    Value::Number(v)
                }
                NumberConfig::Decimal { default, .. } => {
                    // Parse as f64 for decimal
                    let num_opt = if new_value.trim().is_empty() {
                        None
                    } else {
                        Some(
                            new_value
                                .parse::<f64>()
                                .map_err(|_| ServerFnError::new("Invalid decimal format"))?,
                        )
                    };
                    use charac::models::record::cell::DecimalValue;
                    let v = DecimalValue::new(num_opt, default.map(|f| f as f64))
                        .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
                    Value::Decimal(v)
                }
                _ => {
                    // Fallback for other number types - parse as usize
                    let num_opt = if new_value.trim().is_empty() {
                        None
                    } else {
                        Some(
                            new_value
                                .parse::<usize>()
                                .map_err(|_| ServerFnError::new("Invalid number format"))?,
                        )
                    };
                    let v = NumberValue::new(num_opt, None)
                        .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
                    Value::Number(v)
                }
            }
        }
        FieldConfig::Select(_) => {
            // Store select values as SingleLine for now
            let v = SingleLineValue::new(None, value_opt)
                .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
            Value::SingleLine(v)
        }
        FieldConfig::Datetime(_) => {
            // Parse datetime from string
            use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone, Utc};
            use std::str::FromStr;

            let dt = if new_value.len() == 10 {
                // Try parsing as YYYY-MM-DD (standard browser date input)
                NaiveDate::parse_from_str(&new_value, "%Y-%m-%d")
                    .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                    .map(|d| Utc.from_utc_datetime(&d))
                    .map_err(|_| ServerFnError::new("Invalid date format. Use YYYY-MM-DD."))?
            } else {
                DateTime::<FixedOffset>::from_str(&new_value)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|_| {
                        ServerFnError::new("Invalid datetime format. Use ISO 8601 format.")
                    })?
            };

            use charac::models::record::cell::DateValue;
            Value::Date(DateValue::new(dt.into()))
        }
        FieldConfig::Relation(_) => {
            // Store relation as SingleLine (RecordId string)
            let v = SingleLineValue::new(None, value_opt)
                .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
            Value::SingleLine(v)
        }
        FieldConfig::User(_) => {
            let v = SingleLineValue::new(None, value_opt)
                .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
            Value::SingleLine(v)
        }
        FieldConfig::Computed(_) => {
            return Err(ServerFnError::new("Cannot edit computed fields"));
        }
        FieldConfig::Custom(_) => {
            let v = SingleLineValue::new(None, value_opt)
                .map_err(|e| ServerFnError::new(format!("Validation failed: {e}")))?;
            Value::SingleLine(v)
        }
    };

    let cell_value = CellValue::new(value);
    let record_record_id = RecordId::parse_simple(&record_id)
        .map_err(|_| ServerFnError::new("Couldn't parse the record id"))?;

    use surrealdb::types::ToSql;
    let field_id_str = field
        .id
        .ok_or(ServerFnError::new("Field ID missing"))?
        .0
        .to_sql();

    table_service
        .update_record(
            CharacRecordId(record_record_id),
            RecordPatch::new(Some(vec![(field_id_str, cell_value)])),
        )
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let duration = start.elapsed().as_millis();
    println!("[update_record_cell] finished in {}ms", duration);

    Ok(())
}

#[server]
pub async fn create_field(
    base_id: String,
    table_id: String,
    name: String,
    kind: Option<String>,
) -> Result<TableField, ServerFnError> {
    use charac::models::field::{FieldConfig, InsertField, NumberConfig, TextConfig};
    use charac::models::ids::{BaseId, TableId};
    use std::time::Instant;
    use surrealdb::types::{RecordId, ToSql};

    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let mut user_service = service;

    let base_record_id = if base_id.contains(':') {
        RecordId::parse_simple(&base_id)
    } else {
        RecordId::parse_simple(format!("base:{}", base_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the base id"))?;
    let base_id_typed = BaseId(base_record_id);

    user_service
        .open_base(base_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;

    let table_record_id = if table_id.contains(':') {
        RecordId::parse_simple(&table_id)
    } else {
        RecordId::parse_simple(format!("table:{}", table_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the table id"))?;

    let table_id_typed = TableId(table_record_id);

    let table_service = user_service
        .current_base
        .as_ref()
        .unwrap()
        .open_table(table_id_typed.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    // Determine config based on kind
    let config = match kind.as_deref() {
        Some("Number") => FieldConfig::Number(NumberConfig::Number { default: None }),
        Some("Email") => FieldConfig::Text(TextConfig::Email),
        Some("URL") => FieldConfig::Text(TextConfig::URL),
        Some("Phone") => FieldConfig::Text(TextConfig::Phone),
        Some("LongText") => FieldConfig::Text(TextConfig::LongText { rich_text: false }),
        Some("Date") => {
            use charac::models::field::{DateFormat, DatetimeConfig};
            FieldConfig::Datetime(DatetimeConfig::Date {
                format: DateFormat::ISO,
                include_time: false,
            })
        }
        _ => FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 255,
        }),
    };

    let insert_field = InsertField::new(
        name.clone(),
        config.clone(),
        false, // is_primary
        true,  // is_nullable
        false, // is_unique
    );

    let field = table_service
        .create_field(insert_field)
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let duration = start.elapsed().as_millis();
    println!("[create_field] finished in {}ms", duration);

    Ok(TableField {
        id: field.id.unwrap().0.to_sql(),
        name,
        config,
    })
}

#[server]
pub async fn rename_field(
    base_id: String,
    table_id: String,
    field_id: String,
    new_name: String,
) -> Result<(), ServerFnError> {
    use charac::db::DB;
    use charac::models::field::Field;
    use charac::models::ids::{BaseId, TableId};
    use std::time::Instant;
    use surrealdb::types::RecordId;

    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let mut user_service = service;
    dbg!(&base_id, &table_id, &field_id);

    let base_record_id = RecordId::parse_simple(format!("base:{}", base_id).as_str())
        .ok()
        .ok_or(ServerFnError::new("Couldn't parse the base id"))?;
    let base_id_typed = BaseId(base_record_id);

    user_service
        .open_base(base_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;

    let table_record_id = RecordId::parse_simple(&table_id)
        .ok()
        .ok_or(ServerFnError::new("Couldn't parse the table id"))?;

    let table_id_typed = TableId(table_record_id);

    let _table_service = user_service
        .current_base
        .as_ref()
        .unwrap()
        .open_table(table_id_typed.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let field_record_id = RecordId::parse_simple(&field_id)
        .map_err(|_| ServerFnError::new("Couldn't parse the field id"))?;

    let mut res = DB
        .query("SELECT * FROM $field WHERE table = $table AND is_deleted = false")
        .bind(("field", field_record_id.clone()))
        .bind(("table", table_id_typed))
        .await?;
    let _field: Field = res
        .take::<Option<Field>>(0)?
        .ok_or(ServerFnError::new("Field not found"))?;
    dbg!(&new_name);

    let res = DB
        .query("UPDATE $field SET name = $name")
        .bind(("field", field_record_id))
        .bind(("name", new_name))
        .await?;
    dbg!(res);

    let duration = start.elapsed().as_millis();
    println!("[rename_field] finished in {}ms", duration);

    Ok(())
}

#[server]
pub async fn delete_field(
    base_id: String,
    table_id: String,
    field_id: String,
) -> Result<(), ServerFnError> {
    use charac::models::ids::{BaseId, FieldId, TableId};
    use std::time::Instant;
    use surrealdb::types::RecordId;

    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let mut user_service = service;

    let base_record_id = if base_id.contains(':') {
        RecordId::parse_simple(&base_id)
    } else {
        RecordId::parse_simple(format!("base:{}", base_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the base id"))?;
    let base_id_typed = BaseId(base_record_id);

    user_service
        .open_base(base_id_typed)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;

    let table_record_id = if table_id.contains(':') {
        RecordId::parse_simple(&table_id)
    } else {
        RecordId::parse_simple(format!("table:{}", table_id).as_str())
    }
    .ok()
    .ok_or(ServerFnError::new("Couldn't parse the table id"))?;

    let table_id_typed = TableId(table_record_id);

    let table_service = user_service
        .current_base
        .as_ref()
        .unwrap()
        .open_table(table_id_typed.clone())
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let field_record_id = RecordId::parse_simple(&field_id)
        .map_err(|_| ServerFnError::new("Couldn't parse the field id"))?;

    table_service
        .delete_field(FieldId(field_record_id))
        .await
        .map_err(|e| ServerFnError::new(format!("{e:?}")))?;

    let duration = start.elapsed().as_millis();
    println!("[delete_field] finished in {}ms", duration);

    Ok(())
}

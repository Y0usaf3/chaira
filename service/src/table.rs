use crate::kinds::FieldConfig;
use crate::migration::MigrationStrategy;
use crate::prelude::*;
use std::time::Instant;

// TODO: gotta work here :sob:
// fr :noooooovanish:
//
// TODO: make functions to get the list of fields and another to get the entire table data
//
// fuck it fuck it we're REWRITING the damn table service, and the tables too, BECAUSE i suck at
// making it fast
// first of all im going to research how airtable stores their data
// https://databasesample.com/blog/airtable-sql
//
// TODO: cache the fields ??
//
// not sure abt that actually, maybe
//
// well no if we do that itd be hard to see the changes done by others instantly

#[derive(Debug, Clone)]
struct StateCache {
    is_owner: bool,
    permissions: TablePermissions,
}

#[derive(Debug, Clone)]
pub struct TableService {
    pub table: Table,
    pub user: UserId,
    pub base: BaseId,
    table_record_id: TableId,
    cache: Option<StateCache>,
    cache_instant: Option<Instant>,
}

// NOTE: FR stands for frontend :p
//
// ok stupid yousafe, WE DONT EVEN NEED A SEPARATE STRUCT FUCK U
//
// ok no actually this yousafe was smart
#[derive(Serialize, Deserialize, SurrealValue)]
pub struct FieldConfigFR {
    pub is_deleted: bool,
    pub config: FieldConfig,
    pub is_primary: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub name: String,
    pub description: Option<String>,
}

pub struct PaginationParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub can_migrate: bool,
    pub success_rate: f32,
    pub affected_records: usize,
    pub failed_records: usize,
    pub warning: Option<String>,
}

impl TableService {
    pub async fn new(tablee: TableId, base: BaseId, user: UserId) -> Result<Self, Irror> {
        let mut res = DB.query("
            LET $is_owner = (SELECT VALUE owner FROM $base)[0] == $user;
            
            SELECT * FROM $table_id WHERE is_deleted = false AND (
                $is_owner OR 
                fn::can(
                    (SELECT VALUE perms FROM can_access_table WHERE in = $user AND out = $this.id)[0], 
                    2
                )
            );
        ")
        .bind(("user", user.clone()))
        .bind(("base", base.clone()))
        .bind(("table_id", tablee.clone()))
        .await?;

        let table: Table = res.take::<Option<Table>>(1)?.ok_or(TableError::NotFound)?;

        Ok(Self {
            table,
            user,
            base,
            table_record_id: tablee,
            cache: None,
            cache_instant: None,
        })
    }

    async fn load_state(&mut self) -> Result<StateCache, Irror> {
        if let Some((value, ts)) = self.cache.clone().zip(self.cache_instant)
            && ts.elapsed() < Duration::from_secs(1)
        {
            return Ok(value);
        };

        let mut res = DB
            .query(
                "(SELECT VALUE owner FROM $target_base)[0] == $user;
                (SELECT VALUE perms FROM can_access_table WHERE in = $user AND out = $target_table)[0] OR 0;",
            )
            .bind(("user", self.user.clone()))
            .bind(("target_table", self.table_record_id.clone()))
            .bind(("target_base", self.base.clone()))
            .await?;
        let is_owner = res.take::<Option<bool>>(0)?.unwrap_or(false);
        let permissions = res
            .take::<Option<TablePermissions>>(1)?
            .unwrap_or(TablePermissions::from(0));
        let value = StateCache {
            is_owner,
            permissions,
        };

        self.cache = Some(value.clone());
        self.cache_instant = Some(Instant::now());
        Ok(value)
    }

    pub async fn get_field_config(&mut self, field_id: FieldId) -> Result<FieldConfigFR, Irror> {
        let state = self.load_state().await?;
        let mut res = DB
            .query(
                "
            SELECT * FROM $field 
            WHERE 
                table = $table_id AND 
                table.base = $base_id AND 
                is_deleted = false AND
                (
                    $is_owner OR
                    fn::can(
                        $permissions, 
                        2
                    )
                )
        ",
            )
            .bind(("field", field_id))
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("base_id", self.base.clone()))
            .bind(("is_owner", state.is_owner))
            .bind(("permissions", state.permissions))
            .await?;

        let field_config: Option<FieldConfigFR> = res.take(0)?;

        match field_config {
            Some(config) => Ok(config),
            None => Err(Irror::Table(TableError::NotFound)),
        }
    }

    #[requires(TablePermission, Edit)]
    pub async fn create_field(&mut self, field: InsertField) -> Result<Field, Irror> {
        let field = Field::from_insert(field);
        let mut res = DB
            .query(
                "CREATE field SET 
                    name = $data.name,
                    table = $table_id,
                    is_primary = $data.is_primary,
                    is_nullable = $data.is_nullable,
                    is_unique = $data.is_unique,
                    order = $data.order,
                    description = $data.description,
                    config = $data.config,
                    created_at = time::now(),
                    updated_at = time::now();",
            )
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("data", field))
            .await?;

        let created_field: Option<Field> = res.take(0)?;

        match created_field {
            Some(f) => Ok(f),
            None => Err(Irror::Table(TableError::CreateFailed)),
        }
    }

    #[requires(TablePermission, Edit)]
    pub async fn create_a_lot_of_fields(
        &mut self,
        fields: Vec<InsertField>,
    ) -> Result<Vec<Field>, Irror> {
        if fields.len() >= 100 {
            return Err(Irror::Table(TableError::Unauthorized));
        };
        let fields_data: Vec<Field> = fields.into_iter().map(Field::from_insert).collect();

        let mut res = DB
            .query(
                "
        (INSERT INTO field (
            SELECT 
                name,
                $table_id AS table,
                is_primary,
                is_nullable,
                is_unique,
                order,
                description,
                config,
                time::now() AS created_at,
                time::now() AS updated_at
            FROM $data
        ))",
            )
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("data", fields_data))
            .await?;

        let created_fields: Vec<Field> = res.take(0)?;

        if created_fields.is_empty() {
            Err(Irror::Table(TableError::NotFound))
        } else {
            Ok(created_fields)
        }
    }

    #[requires(TablePermission, Edit)]
    pub async fn update_field(
        &mut self,
        field_id: FieldId,
        field: InsertField,
    ) -> Result<Result<Field, MigrationStrategy>, Irror> {
        let field = Field::from_insert(field);
        let mut res = DB
            .query("SELECT * FROM $field WHERE table = $table_id")
            .bind(("field", field_id.clone()))
            .bind(("table_id", self.table_record_id.clone()))
            .await?;

        let current_field: Field = res
            .take::<Option<Field>>(0)?
            .ok_or(Irror::Table(TableError::NotFound))?;

        let strategy = current_field.config.get_migration_strategy(&field.config);

        if strategy == MigrationStrategy::Risky || strategy == MigrationStrategy::Destructive {
            return Ok(Err(strategy));
        }

        let mut update_res = DB
            .query("UPDATE $field CONTENT $field_config")
            .bind(("field", field_id))
            .bind(("field_config", field))
            .await?;

        let updated: Field = update_res
            .take::<Option<Field>>(0)?
            .ok_or(Irror::Table(TableError::NotFound))?;

        Ok(Ok(updated))
    }

    #[requires(TablePermission, Edit)]
    pub async fn delete_field(&mut self, field: FieldId) -> Result<Field, Irror> {
        let mut res = DB
            .query(
                "
        UPDATE $field SET 
            is_deleted = true,
            updated_at = time::now();
        ",
            )
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("field", field))
            .await?;
        let deleted_field: Option<Field> = res.take(0)?;

        match deleted_field {
            Some(f) => Ok(f),
            _ => Err(Irror::Table(TableError::DeleteFailed)),
        }
    }

    #[requires(TablePermission, Edit)]
    pub async fn delete_a_lot_of_fields(
        &mut self,
        fields: Vec<FieldId>,
    ) -> Result<Vec<Field>, Irror> {
        let mut res = DB
            .query(
                "
        (UPDATE field SET 
            is_deleted = true,
            updated_at = time::now()
        WHERE id IN $fields)
        ",
            )
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("fields", fields))
            .await?;
        let delected_fields: Option<Vec<Field>> = res.take(0)?;
        match delected_fields {
            Some(f) => Ok(f),
            _ => Err(Irror::Table(TableError::DeleteFailed)),
        }
    }

    pub async fn get_record(&mut self, record_id: RecordId) -> Result<Record, Irror> {
        let state = self.load_state().await?;
        let mut res = DB
            .query(
                "
        SELECT * FROM $record_id 
        WHERE 
            table = $table_id AND 
            is_deleted = false AND
            (
                $is_owner OR
                fn::can(
                    $perms,
                    2
                )
            )
    ",
            )
            .bind(("record_id", record_id))
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("user", self.user.clone()))
            .bind(("is_owner", state.is_owner))
            .bind(("perms", state.permissions))
            .await?;

        let record: Option<Record> = res.take(0)?;

        match record {
            Some(r) => Ok(r),
            None => Err(Irror::Table(TableError::NotFound)),
        }
    }

    pub async fn list_records(
        &mut self,
        pagination_params: PaginationParams,
    ) -> Result<Vec<Record>, Irror> {
        let limit = pagination_params.limit.unwrap_or(10);
        let skip = pagination_params.offset.unwrap_or(0);
        let state = self.load_state().await?;

        let mut res = DB
            .query(
                "SELECT * FROM record 
         WHERE 
             table = $table_id AND 
             is_deleted = false AND
             ($is_owner OR fn::can($perms, 2))
         ORDER BY created_at ASC
         LIMIT $limit
         START $skip;",
            )
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("limit", limit))
            .bind(("skip", skip))
            .bind(("is_owner", state.is_owner))
            .bind(("perms", state.permissions))
            .await?;
        let records: Vec<Record> = res.take(0)?;

        Ok(records)
    }

    pub async fn get_full_data(
        &mut self,
        limit: Option<u32>,
    ) -> Result<(Vec<Field>, Vec<Record>), Irror> {
        let limit = limit.unwrap_or(50);
        let state = self.load_state().await?;
        let mut res = DB
            .query(
                "
                IF !$is_owner AND !fn::can($perms, 2) {
                    THROW 'Permission Denied';
                };

                SELECT * FROM field WHERE table = $table_id AND is_deleted = false ORDER BY created_at ASC;
                SELECT * FROM record WHERE table = $table_id AND is_deleted = false ORDER BY created_at ASC LIMIT $limit;
            ",
            )
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("user", self.user.clone()))
            .bind(("limit", limit))
            .bind(("is_owner", state.is_owner))
            .bind(("perms", state.permissions))
            .await?;

        let fields: Vec<Field> = res.take(1)?;
        let records: Vec<Record> = res.take(2)?;

        Ok((fields, records))
    }

    #[requires(TablePermission, Edit)]
    pub async fn create_record(&mut self, record: InsertRecord) -> Result<Record, Irror> {
        let record = Record::from_insert(record);

        let mut res = DB
            .query(
                "CREATE record SET 
                    table = $table_id,
                    cells = $data.cells,
                    is_deleted = false,
                    created_at = time::now(),
                    updated_at = time::now();",
            )
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("data", record))
            .await?;

        let created_records: Option<Record> = res.take(0)?;

        match created_records {
            Some(r) => Ok(r),
            None => Err(Irror::Table(TableError::CreateFailed)),
        }
    }

    #[requires(TablePermission, Edit)]
    pub async fn create_a_lot_of_records(
        &mut self,
        records: Vec<InsertRecord>,
    ) -> Result<(), Irror> {
        // Convert all records once, then batch insert
        let records_data: Vec<Record> = records.into_iter().map(Record::from_insert).collect();

        for chunk in records_data.chunks(5000) {
            let mut batch_records = Vec::new();
            for record in chunk {
                let mut new_record = record.clone();
                new_record.table = self.table_record_id.clone();
                batch_records.push(new_record);
            }

            let res = DB
                .query("INSERT INTO record (SELECT * FROM $data);")
                .bind(("data", batch_records))
                .await?;

            res.check()?;
        }
        Ok(())
    }

    #[requires(TablePermission, Edit)]
    pub async fn update_record(
        &mut self,
        record_id: RecordId,
        patch: RecordPatch,
    ) -> Result<Record, Irror> {
        let (cells_map, has_cells) = if let Some(changed_cells) = patch.changed_cells {
            let mut map = std::collections::HashMap::new();
            for (key, value) in changed_cells {
                map.insert(key, value);
            }
            (Some(map), true)
        } else {
            (None, false)
        };

        let query_str = if has_cells {
            "UPDATE $record_id SET cells = object::extend(cells, $cells), updated_at = time::now();"
        } else {
            "UPDATE $record_id SET updated_at = time::now();"
        };

        let mut query = DB.query(query_str).bind(("record_id", record_id));

        if let Some(cells) = cells_map {
            query = query.bind(("cells", cells));
        }

        let mut res = query.await?;
        let updated: Option<Record> = res.take(0)?;

        match updated {
            Some(r) => Ok(r),
            None => Err(Irror::Table(TableError::NotFound)),
        }
    }

    #[requires(TablePermission, Edit)]
    pub async fn delete_record(&mut self, record_id: RecordId) -> Result<Record, Irror> {
        let mut res = DB
            .query("UPDATE $record_id SET is_deleted = true, updated_at = time::now();")
            .bind(("record_id", record_id))
            .await?;

        let deleted_record: Option<Record> = res.take(0)?;

        match deleted_record {
            Some(r) => Ok(r),
            None => Err(Irror::Table(TableError::NotFound)),
        }
    }

    #[requires(TablePermission, View)]
    pub async fn check_migration(
        &mut self,
        field_id: FieldId,
        target_config: FieldConfig,
    ) -> Result<MigrationReport, Irror> {
        let mut field_res = DB
            .query("SELECT VALUE name FROM $field_id WHERE table = $table_id")
            .bind(("field_id", field_id))
            .bind(("table_id", self.table_record_id.clone()))
            .await?;
        let field_name: String = field_res
            .take::<Option<String>>(0)?
            .ok_or(Irror::Table(TableError::NotFound))?;

        let mut record_res = DB
            .query("SELECT * FROM record WHERE table = $table_id AND is_deleted = false")
            .bind(("table_id", self.table_record_id.clone()))
            .await?;
        let records: Vec<Record> = record_res.take(0)?;

        let total = records.len();
        let mut successful = 0;

        for record in &records {
            if let Some(cell) = record.cells.get(&field_name) {
                if cell.value.convert_to(&target_config).is_ok() {
                    successful += 1;
                }
            } else {
                successful += 1;
            }
        }

        let failed = total - successful;
        let success_rate = if total == 0 {
            1.0
        } else {
            successful as f32 / total as f32
        };

        Ok(MigrationReport {
            can_migrate: success_rate > 0.5,
            success_rate,
            affected_records: total,
            failed_records: failed,
            warning: if success_rate < 1.0 {
                Some(format!(
                    "{} out of {} records will fail conversion",
                    failed, total
                ))
            } else {
                None
            },
        })
    }

    #[requires(TablePermission, View)]
    pub async fn migrate_field_type(
        &mut self,
        field_id: FieldId,
        new_config: FieldConfig,
    ) -> Result<Result<Field, String>, Irror> {
        let mut field_res = DB
            .query("SELECT * FROM $field_id WHERE table = $table_id")
            .bind(("field_id", field_id.clone()))
            .bind(("table_id", self.table_record_id.clone()))
            .await?;
        let current_field: Field = field_res
            .take::<Option<Field>>(0)?
            .ok_or(Irror::Table(TableError::NotFound))?;
        let strategy = current_field.config.get_migration_strategy(&new_config);

        if strategy == MigrationStrategy::Risky || strategy == MigrationStrategy::Destructive {
            return Ok(Err(format!(
                "Migration is {:?}. Data loss might occur.",
                strategy
            )));
        }
        if strategy == MigrationStrategy::NoOp {
            return Ok(Err("No operation needed".to_string()));
        }

        let mut record_res = DB
            .query("SELECT * FROM record WHERE table = $table_id AND is_deleted = false")
            .bind(("table_id", self.table_record_id.clone()))
            .await?;
        let records: Vec<Record> = record_res.take(0)?;

        for record in records {
            if let Some(cell) = record.cells.get(&current_field.name)
                && let Ok(new_value) = cell.value.convert_to(&new_config)
            {
                let mut new_cell = cell.clone();
                new_cell.value = new_value;
                new_cell.updated_at = Datetime::now();

                DB.query("UPDATE $record_id SET cells[$field_name] = $new_cell")
                    .bind(("record_id", record.id.clone()))
                    .bind(("field_name", current_field.name.clone()))
                    .bind(("new_cell", new_cell))
                    .await?;
            }
        }

        let mut update_res = DB
            .query("UPDATE $field_id SET config = $new_config, updated_at = time::now()")
            .bind(("field_id", field_id))
            .bind(("new_config", new_config))
            .await?;

        let updated: Field = update_res
            .take::<Option<Field>>(0)?
            .ok_or(Irror::Table(TableError::UpdateFailed))?;
        Ok(Ok(updated))
    }

    pub async fn force_edit_field_config(
        &mut self,
        field_id: FieldId,
        new_config: FieldConfig,
    ) -> Result<Field, Irror> {
        // TODO: add a permission to force edit fields
        // im pretty sure theyd like to stop ppl from editing the whole thing lol
        let state = self.load_state().await?;
        if !state.is_owner {
            return Err(Irror::Table(TableError::Unauthorized));
        };

        let mut res = DB
            .query(
                "
        UPDATE $field_id SET 
            config = $new_config,
            updated_at = time::now()
        WHERE table = $table_id AND is_deleted = false
    ",
            )
            .bind(("field_id", field_id))
            .bind(("table_id", self.table_record_id.clone()))
            .bind(("new_config", new_config))
            .await?;

        let updated: Option<Field> = res.take(0)?;

        match updated {
            Some(f) => Ok(f),
            None => Err(Irror::Table(TableError::NotFound)),
        }
    }
}

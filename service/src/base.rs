use crate::prelude::*;
use crate::table::TableService;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct StateCache {
    permissions: BasePermissions,
    is_owner: bool,
}

#[derive(Debug, Clone)]
pub struct BaseService {
    pub base: Base,
    pub user: UserId,
    base_record_id: BaseId,
    //  pub current_table: Option<TableService>,
    cache: Option<StateCache>,
    cache_instant: Option<Instant>,
}

impl BaseService {
    pub fn id(&self) -> &BaseId {
        &self.base_record_id
    }

    pub async fn new(base_id: BaseId, user: UserId) -> Result<Self, Irror> {
        let mut res = DB
            .query(
                "
           BEGIN TRANSACTION;
            SELECT * FROM $base 
            WHERE is_deleted = false AND (
                owner = $user OR 
                fn::can((SELECT VALUE perms FROM can_access_base WHERE in = $user AND out = $this.id)[0], 2)
            )
            LIMIT 1;
            COMMIT TRANSACTION;",
            )
            .bind(("base", base_id.clone()))
            .bind(("user", user.clone()))
            .await?;
        let base: Base = res.take::<Option<Base>>(1)?.ok_or(BaseError::NotFound)?;
        Ok(Self {
            base,
            base_record_id: base_id,
            user,
            //  current_table: None,
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
                "(SELECT VALUE owner FROM $base)[0] == $user;
                (SELECT VALUE perms FROM can_access_base WHERE in = $user AND out = $base)[0];",
            )
            .bind(("user", self.user.clone()))
            .bind(("base", self.base_record_id.clone()))
            .await?;
        let is_owner = res.take::<Option<bool>>(0)?.unwrap_or(false);
        let permissions = res
            .take::<Option<BasePermissions>>(1)?
            .unwrap_or(BasePermissions::from(0));
        let value = StateCache {
            is_owner,
            permissions,
        };

        self.cache = Some(value.clone());
        self.cache_instant = Some(Instant::now());
        Ok(value)
    }

    #[requires(BasePermission, ManageInvitations)]
    pub async fn invite_user(&mut self, user: UserId, perms: BasePermissions) -> Result<(), Irror> {
        let res = DB
            .query(
                "
            BEGIN TRANSACTION;

RELATE $invited_id->can_access_base->$target_base 
    SET perms = $perms;

COMMIT TRANSACTION;
            ",
            )
            .bind(("invited_id", user))
            .bind(("target_base", self.base_record_id.clone()))
            .bind(("perms", perms))
            .await?;
        res.check()?;
        Ok(())
    }

    #[requires(BasePermission, Delete)]
    pub async fn delete(&mut self) -> Result<Base, Irror> {
        let mut res = DB
            .query(
                "
        BEGIN TRANSACTION;

        UPDATE $base SET 
            is_deleted = true, 
            updated_at = time::now();

        COMMIT TRANSACTION;
    ",
            )
            .bind(("base", self.base_record_id.clone()))
            .await?;

        let base: Option<Base> = res.take(2)?;
        let base = base.ok_or(BaseError::DeleteFailed)?;

        Ok(base)
    }

    #[requires(BasePermission, ManageTables)]
    pub async fn create_table(&mut self, name: String) -> Result<Table, Irror> {
        approved(&name)?;

        use std::collections::HashMap;

        let field = Field::from_insert(InsertField {
            name: "Meow_colon_three".to_string(),
            description: None,
            is_primary: false,
            is_nullable: false,
            is_unique: false,
            order: 0,
            config: FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 500,
            }),
        });

        let mut cells = HashMap::new();
        cells.insert(
            "Name".to_string(),
            Value::SingleLine(SingleLineValue::new(
                None,
                Some("Miniwi :3".into()),
            ).unwrap()),
        );

        let mut res = DB
            .query(
                "
            BEGIN TRANSACTION;

            LET $tablee = (CREATE table SET 
                name = $name, 
                base = $base, 
                is_deleted = false
            );

            RELATE $user->can_access_table->$tablee SET perms = 7;

            LET $field = (CREATE field SET 
                name = $field_data.name,
                table = $tablee[0].id,
                is_primary = $field_data.is_primary,
                is_nullable = $field_data.is_nullable,
                is_unique = $field_data.is_unique,
                order = $field_data.order,
                description = $field_data.description,
                config = $field_data.config,
                created_at = time::now(),
                updated_at = time::now()
            );

            CREATE record SET 
                table = $tablee[0].id,
                cells = $cells,
                cell_metadata = {},
                version = 1,
                is_deleted = false,
                created_at = time::now(),
                updated_at = time::now();

            RETURN $tablee;

            COMMIT TRANSACTION;
        ",
            )
            .bind(("user", self.user.clone()))
            .bind(("base", self.base_record_id.clone()))
            .bind(("name", name))
            .bind(("field_data", field))
            .bind(("cells", cells))
            .await?;

        let table = res.take::<Vec<Table>>(5)?;
        if table.is_empty() {
            return Err(Irror::Table(TableError::CreateFailed));
        };
        Ok(table[0].clone())
    }

    #[requires(BasePermission, ManageTables)]
    pub async fn delete_table(&mut self, table_id: TableId) -> Result<(), Irror> {
        let res = DB
            .query(
                "
            BEGIN TRANSACTION;

            UPDATE $table_id SET is_deleted = true, updated_at = time::now();
            
            UPDATE record SET is_deleted = true WHERE table = $table_id;

            COMMIT TRANSACTION;
        ",
            )
            .bind(("table_id", table_id))
            .await?;

        res.check()?;
        Ok(())
    }

    pub async fn list_tables(&mut self) -> Result<Vec<Table>, Irror> {
        let state = self.load_state().await?;
        let mut res = DB
            .query(
                "
            SELECT * FROM table WHERE base = $base AND is_deleted = false AND (
                $is_owner OR 
                fn::can(
                    $perms, 
                    2
                )
            ) ORDER BY created_at ASC;
        ",
            )
            .bind(("user", self.user.clone()))
            .bind(("base", self.base_record_id.clone()))
            .bind(("is_owner", state.is_owner))
            .bind(("perms", state.permissions))
            .await?;

        let tables: Vec<Table> = res.take(0)?;
        Ok(tables)
    }

    pub async fn open_table(&self, table_id: TableId) -> Result<TableService, Irror> {
        let service =
            TableService::new(table_id, self.base_record_id.clone(), self.user.clone()).await?;

        Ok(service)
    }
}

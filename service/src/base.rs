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
    pub current_table: Option<TableService>,
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
            current_table: None,
            cache: None,
            cache_instant: None,
        })
    }

    async fn load_state(&mut self) -> Result<StateCache, Irror> {
        if let Some((value, ts)) = self.cache.clone().zip(self.cache_instant)
            && ts.elapsed() < Duration::from_secs(5)
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

    pub async fn invite_user(&mut self, user: UserId, perms: BasePermissions) -> Result<(), Irror> {
        let state = self.load_state().await?;
        let res = DB
            .query(
                "
            BEGIN TRANSACTION;

-- View (1 << 1) = 2 + ManageInvitations (1 << 8) = 256
IF !$is_owner AND !fn::can($inviter_perms, 258) {
    THROW 'Unauthorized: You need [View] and [ManageInvitations] to invite others.';
};

RELATE $invited_id->can_access_base->$target_base 
    SET perms = $perms;

COMMIT TRANSACTION;
            ",
            )
            .bind(("inviter_id", self.user.clone()))
            .bind(("invited_id", user))
            .bind(("target_base", self.base_record_id.clone()))
            .bind(("perms", perms))
            .bind(("is_owner", state.is_owner))
            .bind(("inviter_perms", state.permissions))
            .await?;
        res.check()?;
        Ok(())
    }

    pub async fn delete(&mut self) -> Result<Base, Irror> {
        let state = self.load_state().await?;
        let mut res = DB
            .query(
                "
        BEGIN TRANSACTION;

        -- 'Delete' (1 << 3 = 8) 
        
        LET $is_admin = (SELECT VALUE role FROM $user WHERE id = $user)[0] == 'admin';
        
        IF !$is_owner AND !$is_admin AND !fn::can($user_perms, 8) {
            THROW 'Unauthorized: You do not have permission to delete this base.';
        };

        UPDATE $base SET 
            is_deleted = true, 
            updated_at = time::now();

        COMMIT TRANSACTION;
    ",
            )
            .bind(("base", self.base_record_id.clone()))
            .bind(("user", self.user.clone()))
            .bind(("is_owner", state.is_owner))
            .bind(("user_perms", state.permissions))
            .await?;

        let base: Option<Base> = res.take(3)?;
        let base = base.ok_or(BaseError::DeleteFailed)?;

        Ok(base)
    }

    pub async fn create_table(&mut self, name: String) -> Result<Table, Irror> {
        approved(&name)?;
        let state = self.load_state().await?;
        let mut res = DB
            .query(
                "
            BEGIN TRANSACTION;
            IF !$is_owner AND !fn::can($user_perms, 16) {
                THROW 'Unauthorized: You do not have ManageTables permission.';
            };

            -- Create the table linked to this base
            LET $table = (CREATE table SET 
                name = $name, 
                base = $base, 
                is_deleted = false
            );

            -- Automatically grant creator 'Full Access' (1|2|4 = 7) to this specific table
            RELATE $user->can_access_table->$table SET perms = 7;

            RETURN $table;

            COMMIT TRANSACTION;
        ",
            )
            .bind(("user", self.user.clone()))
            .bind(("base", self.base_record_id.clone()))
            .bind(("name", name))
            .bind(("is_owner", state.is_owner))
            .bind(("user_perms", state.permissions))
            .await?;

        let table = res.take::<Vec<Table>>(4)?;
        if table.is_empty() {
            return Err(Irror::Table(TableError::CreateFailed));
        };
        Ok(table[0].clone())
    }

    pub async fn delete_table(&mut self, table_id: TableId) -> Result<(), Irror> {
        let state = self.load_state().await?;
        let res = DB.query("
            BEGIN TRANSACTION;
            LET $table_perms = (SELECT VALUE perms FROM can_access_table WHERE in = $user AND out = $table_id)[0] OR 0;

            IF !$is_owner AND !fn::can($base_perms, 16) AND !mod::bit::can($table_perms, 4) {
                THROW 'Unauthorized: Cannot delete this table.';
            };

            UPDATE $table_id SET is_deleted = true, updated_at = time::now();
            
            UPDATE record SET is_deleted = true WHERE table = $table_id;

            COMMIT TRANSACTION;
        ")
        .bind(("user", self.user.clone()))
        .bind(("base", self.base_record_id.clone()))
        .bind(("table_id", table_id))
        .bind(("is_owner", state.is_owner))
        .bind(("base_perms", state.permissions))
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

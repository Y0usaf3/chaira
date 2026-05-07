use axum::{
    Json, Router,
    extract::Path,
    routing::{delete, get, patch, post},
};
use charac::db::Irror;
use charac::models::*;
use charac::service::user::UserService;

pub fn router() -> Router<crate::AppState> {
    Router::new()
        // User Service
        .route("/me", get(get_me))
        .route("/me", patch(update_me))
        .route("/me/refresh", post(refresh_me))
        .route("/me/is_admin", get(is_admin))
        .route("/bases", get(list_bases))
        .route("/bases", post(create_base))
        .route("/bases/{id}", delete(delete_base))
        .route("/tokens", post(create_api_token))
        // Base Service
        .route("/bases/{id}/tables", get(list_tables))
        .route("/bases/{id}/tables", post(create_table))
        .route("/bases/{base_id}/tables/{table_id}", delete(delete_table))
        // Table Service
        .route("/tables/{table_id}/data", get(get_table_data))
        .route("/tables/{table_id}/fields", post(create_field))
        .route("/tables/{table_id}/records", get(list_records))
        .route("/tables/{table_id}/records", post(create_record))
        .route("/tables/{table_id}/records/{record_id}", get(get_record))
        .route(
            "/tables/{table_id}/records/{record_id}",
            patch(update_record),
        )
        .route(
            "/tables/{table_id}/records/{record_id}",
            delete(delete_record),
        )
}

async fn get_me(user_service: UserService) -> Json<User> {
    Json(user_service.user)
}

async fn delete_table(
    user_service: UserService,
    Path((base_id, table_id)): Path<(String, String)>,
) -> Result<(), Irror> {
    let base_id = BaseId(
        surrealdb::types::RecordId::parse_simple(format!("base:{}", base_id).as_str()).unwrap(),
    );
    let table_id = TableId(surrealdb::types::RecordId::parse_simple(&table_id).unwrap());

    let base_service =
        charac::service::base::BaseService::new(base_id, user_service.user_record_id.clone())
            .await?;
    base_service.delete_table(table_id).await?;
    Ok(())
}

async fn update_me(
    mut user_service: UserService,
    Json(patch): Json<UserPatch>,
) -> Result<Json<User>, Irror> {
    let user = user_service.update_self_user(patch).await?;
    Ok(Json(user))
}

async fn refresh_me(mut user_service: UserService) -> Result<Json<User>, Irror> {
    let user = user_service.refresh_user().await?;
    Ok(Json(user))
}

async fn is_admin(user_service: UserService) -> Result<Json<bool>, Irror> {
    let admin = user_service.is_admin().await?;
    Ok(Json(admin))
}

async fn list_bases(user_service: UserService) -> Result<Json<Vec<Base>>, Irror> {
    let bases = user_service.list_bases().await?;
    Ok(Json(bases))
}

async fn create_base(
    user_service: UserService,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Base>, Irror> {
    let name = payload["name"]
        .as_str()
        .ok_or_else(|| Irror::Db("Missing name".to_string()))?;
    let base = user_service.create_base(name.to_string()).await?;
    Ok(Json(base))
}

async fn delete_base(user_service: UserService, Path(base_id): Path<BaseId>) -> Result<(), Irror> {
    user_service.delete_base(base_id).await?;
    Ok(())
}

async fn create_api_token(user_service: UserService) -> Result<Json<String>, Irror> {
    let token = user_service.create_api_token().await?;
    Ok(Json(token))
}

async fn list_tables(
    user_service: UserService,
    Path(base_id_raw): Path<String>,
) -> Result<Json<Vec<Table>>, Irror> {
    let base_id = BaseId(
        surrealdb::types::RecordId::parse_simple(format!("base:{}", base_id_raw).as_str()).unwrap(),
    );
    let base_service =
        charac::service::base::BaseService::new(base_id, user_service.user_record_id.clone())
            .await?;
    let tables = base_service.list_tables().await?;
    Ok(Json(tables))
}

async fn create_table(
    user_service: UserService,
    Path(base_id_raw): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<Table>, Irror> {
    let base_id = BaseId(
        surrealdb::types::RecordId::parse_simple(format!("base:{}", base_id_raw).as_str()).unwrap(),
    );

    let name = payload["name"]
        .as_str()
        .ok_or_else(|| Irror::Db("Missing name".to_string()))?;
    let base_service =
        charac::service::base::BaseService::new(base_id, user_service.user_record_id.clone())
            .await?;
    let table = base_service.create_table(name.to_string()).await?;
    Ok(Json(table))
}

async fn get_table_data(
    user_service: UserService,
    Path(table_id_raw): Path<String>,
) -> Result<Json<(Vec<Field>, Vec<Record>)>, Irror> {
    let table_id = TableId(
        surrealdb::types::RecordId::parse_simple(format!("table:{}", table_id_raw).as_str())
            .unwrap(),
    );

    let mut res = charac::db::DB
        .query("SELECT VALUE base FROM $table")
        .bind(("table", table_id.clone()))
        .await?;
    let base_id: BaseId = res
        .take::<Option<BaseId>>(0)?
        .ok_or(Irror::Table(charac::service::errors::TableError::NotFound))?;

    let table_service = charac::service::table::TableService::new(
        table_id,
        base_id,
        user_service.user_record_id.clone(),
    )
    .await?;
    let data = table_service.get_full_data(None).await?;
    Ok(Json(data))
}

async fn create_field(
    user_service: UserService,
    Path(table_id_raw): Path<String>,
    Json(field): Json<InsertField>,
) -> Result<Json<Field>, Irror> {
    let table_id = TableId(
        surrealdb::types::RecordId::parse_simple(format!("table:{}", table_id_raw).as_str())
            .unwrap(),
    );

    let mut res = charac::db::DB
        .query("SELECT VALUE base FROM $table")
        .bind(("table", table_id.clone()))
        .await?;
    let base_id: BaseId = res
        .take::<Option<BaseId>>(0)?
        .ok_or(Irror::Table(charac::service::errors::TableError::NotFound))?;

    let table_service = charac::service::table::TableService::new(
        table_id,
        base_id,
        user_service.user_record_id.clone(),
    )
    .await?;
    let field = table_service.create_field(field).await?;
    Ok(Json(field))
}

async fn list_records(
    user_service: UserService,
    Path(table_id_raw): Path<String>,
) -> Result<Json<Vec<Record>>, Irror> {
    let table_id = TableId(
        surrealdb::types::RecordId::parse_simple(format!("table:{}", table_id_raw).as_str())
            .unwrap(),
    );

    let mut res = charac::db::DB
        .query("SELECT VALUE base FROM $table")
        .bind(("table", table_id.clone()))
        .await?;
    let base_id: BaseId = res
        .take::<Option<BaseId>>(0)?
        .ok_or(Irror::Table(charac::service::errors::TableError::NotFound))?;

    let table_service = charac::service::table::TableService::new(
        table_id,
        base_id,
        user_service.user_record_id.clone(),
    )
    .await?;
    let records = table_service
        .list_records(charac::service::table::PaginationParams {
            offset: None,
            limit: None,
        })
        .await?;
    Ok(Json(records))
}

async fn create_record(
    user_service: UserService,
    Path(table_id_raw): Path<String>,
    Json(record): Json<InsertRecord>,
) -> Result<Json<Record>, Irror> {
    let table_id = TableId(
        surrealdb::types::RecordId::parse_simple(format!("table:{}", table_id_raw).as_str())
            .unwrap(),
    );

    let mut res = charac::db::DB
        .query("SELECT VALUE base FROM $table")
        .bind(("table", table_id.clone()))
        .await?;
    let base_id: BaseId = res
        .take::<Option<BaseId>>(0)?
        .ok_or(Irror::Table(charac::service::errors::TableError::NotFound))?;

    let table_service = charac::service::table::TableService::new(
        table_id,
        base_id,
        user_service.user_record_id.clone(),
    )
    .await?;
    let record = table_service.create_record(record).await?;
    Ok(Json(record))
}

async fn get_record(
    user_service: UserService,
    Path((table_id_raw, record_id_raw)): Path<(String, String)>, // Fixed: Combined tuple
) -> Result<Json<Record>, Irror> {
    let table_id = TableId(
        surrealdb::types::RecordId::parse_simple(format!("table:{}", table_id_raw).as_str())
            .unwrap(),
    );
    let record_id = RecordId(
        surrealdb::types::RecordId::parse_simple(format!("record:{}", record_id_raw).as_str())
            .unwrap(),
    );

    let mut res = charac::db::DB
        .query("SELECT VALUE base FROM $table")
        .bind(("table", table_id.clone()))
        .await?;
    let base_id: BaseId = res
        .take::<Option<BaseId>>(0)?
        .ok_or(Irror::Table(charac::service::errors::TableError::NotFound))?;

    let table_service = charac::service::table::TableService::new(
        table_id,
        base_id,
        user_service.user_record_id.clone(),
    )
    .await?;
    let record = table_service.get_record(record_id).await?;
    Ok(Json(record))
}

async fn update_record(
    user_service: UserService,
    Path((table_id_raw, record_id_raw)): Path<(String, String)>,
    Json(patch): Json<RecordPatch>,
) -> Result<Json<Record>, Irror> {
    let table_id = TableId(
        surrealdb::types::RecordId::parse_simple(format!("table:{}", table_id_raw).as_str())
            .unwrap(),
    );
    let record_id = RecordId(
        surrealdb::types::RecordId::parse_simple(format!("record:{}", record_id_raw).as_str())
            .unwrap(),
    );

    let mut res = charac::db::DB
        .query("SELECT VALUE base FROM $table")
        .bind(("table", table_id.clone()))
        .await?;
    let base_id: BaseId = res
        .take::<Option<BaseId>>(0)?
        .ok_or(Irror::Table(charac::service::errors::TableError::NotFound))?;

    let table_service = charac::service::table::TableService::new(
        table_id,
        base_id,
        user_service.user_record_id.clone(),
    )
    .await?;
    let record = table_service.update_record(record_id, patch).await?;
    Ok(Json(record))
}

async fn delete_record(
    user_service: UserService,
    Path((table_id_raw, record_id_raw)): Path<(String, String)>,
) -> Result<Json<Record>, Irror> {
    let table_id = TableId(
        surrealdb::types::RecordId::parse_simple(format!("table:{}", table_id_raw).as_str())
            .unwrap(),
    );
    let record_id = RecordId(
        surrealdb::types::RecordId::parse_simple(format!("record:{}", record_id_raw).as_str())
            .unwrap(),
    );

    let mut res = charac::db::DB
        .query("SELECT VALUE base FROM $table")
        .bind(("table", table_id.clone()))
        .await?;
    let base_id: BaseId = res
        .take::<Option<BaseId>>(0)?
        .ok_or(Irror::Table(charac::service::errors::TableError::NotFound))?;

    let table_service = charac::service::table::TableService::new(
        table_id,
        base_id,
        user_service.user_record_id.clone(),
    )
    .await?;
    let record = table_service.delete_record(record_id).await?;
    Ok(Json(record))
}

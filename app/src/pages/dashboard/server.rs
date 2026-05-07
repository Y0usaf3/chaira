use leptos::prelude::*;

#[derive(serde::Deserialize, serde::Serialize, Clone)]
pub struct UserBase {
    pub name: String,
    pub owner_name: String,
    pub id: String,
}

#[server]
pub async fn create_token() -> Result<String, ServerFnError> {
    let start = std::time::Instant::now();
    let service = crate::get_authenticated_service().await?;
    let raw_token = service.create_api_token().await?;
    let duration = start.elapsed().as_millis();
    println!("created api token in {}ms", duration);
    Ok(raw_token)
}

#[server]
pub async fn get_user_bases() -> Result<Vec<UserBase>, ServerFnError> {
    use std::time::Instant;
    use surrealdb::types::ToSql;
    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;

    let bases = service
        .list_bases()
        .await
        .map_err(|e| ServerFnError::new(format!("Listing Bases failed: {e:?}")))?;

    let user_bases = bases
        .into_iter()
        .map(|b| UserBase {
            name: b.name,
            owner_name: b.owner.0.key.to_sql(),
            id: b.id.unwrap().0.key.to_sql(),
        })
        .collect();
    let duration = start.elapsed().as_millis();
    println!("[get_user_bases] finished in {}ms", duration);
    Ok(user_bases)
}

#[server]
pub async fn create_base(name: String) -> Result<UserBase, ServerFnError> {
    use std::time::Instant;
    let start = Instant::now();
    let service = crate::get_authenticated_service().await?;
    let base = service
        .create_base(name)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    let duration = start.elapsed().as_millis();
    println!("[create_base] finished in {}ms", duration);
    Ok(UserBase {
        name: base.name,
        owner_name: format!("{:?}", base.owner.0.key),
        id: base
            .id
            .map(|id| format!("{:?}", id.0.key))
            .unwrap_or_default(),
    })
}

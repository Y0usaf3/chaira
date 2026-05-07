use crate::db::*;
use crate::models::*;
use crate::service::errors::ApiError;
use crate::service::user::UserService;

#[derive(Debug)]
pub struct ApiService {}

impl ApiService {
    pub async fn get_user(token: String) -> Result<UserService, Irror> {
        let mut res = DB
            .query(
                "SELECT VALUE user FROM api_token 
             WHERE expires_at > time::now() 
             AND crypto::sha512($tokenn) == token 
             LIMIT 1;",
            )
            .bind(("tokenn", token))
            .await?;
        let user_id: UserId = res
            .take::<Option<UserId>>(0)?
            .ok_or(Irror::Api(ApiError::NotFound))?;
        let user: User = DB
            .select::<Option<User>>(user_id.0)
            .await?
            .ok_or(Irror::Api(ApiError::Unauthorized))?;

        Ok(UserService {
            user_record_id: user.id.clone().ok_or(Irror::Api(ApiError::Unauthorized))?,
            user,
            current_base: None,
        })
    }
}

// oookkk thats the most important part now, so for the user i have to know wich functions, bc it
// will be used the most
// so the UserService will have
// login to login the user with an existing acc
// register to create a new user where the email and stuff like that should be unique
// those dont need the self, but for operation that requires self might be to delet a user or smt
// depending on its role so we can update the data, like username etc
// or even deleting the user wich requires admin role
// and in fact, i think workspace user service will be used the most for inside workspace stuff

// oooooooooh shit why did they changed so much stuff iiinn surreeaalllldddbbbb 333...0000
// sooooooooooooooooooooooooooooooon:sob:

// make a function to get record id insteado f having to rerun this shit a million time

use crate::HCAUTH;
use crate::db::*;
use crate::models;
use crate::models::*;
use crate::service::approved;
use crate::service::base::*;
use crate::service::crypter::*;
use crate::service::errors::{AuthError, BaseError, PermissionError, UserError};
use base64::Engine;
use base64::engine::general_purpose;
use chrono::{DateTime, Utc};
use rand::RngExt;
use std::time::{Duration, SystemTime};
use surrealdb::opt::PatchOp;
use surrealdb::types::SurrealValue;

// TODO: add functions to fetch data from HC auth

#[derive(Debug, Clone, PartialEq, SurrealValue, Default)]
pub struct IsAdmin {
    value: bool,
}
impl IsAdmin {
    pub fn value(&self) -> bool {
        self.value
    }
}

pub struct Session {
    pub token: String,
    pub ip: String,
    pub agent: String,
}

pub enum AuthMethod {
    Hca(String),
    Session(Session),
}

#[derive(Debug)]
pub struct UserService {
    pub user: User,
    pub user_record_id: UserId,
    pub current_base: Option<BaseService>,
}

#[cfg(feature = "ssr")]
impl<S> axum::extract::FromRequestParts<S> for UserService
where
    S: Send + Sync,
{
    type Rejection = Irror;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        if let Some(auth_header) = parts.headers.get("Authorization")
        {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    return crate::service::api::ApiService::get_user(token.to_string()).await;
                }
            }
        }

        Err(Irror::Auth(AuthError::InvalidToken))
    }
}

impl UserService {
    pub fn id(&self) -> &UserId {
        &self.user_record_id
    }

    pub async fn login(method: AuthMethod) -> Result<Self, Irror> {
        let user: User = match method {
            AuthMethod::Hca(code) => {
                // NOTE: are we sure hackclub auth is really a secure source, what could go wrong
                // ????
                // [insert what could go wrong here]

                let token = HCAUTH
                    .exchange_code(code)
                    .await
                    .ok()
                    .ok_or(AuthError::VerificationFailed)?;

                let auth_identity = HCAUTH
                    .get_identity(token.access_token.ok_or(AuthError::VerificationFailed)?)
                    .await
                    .map_err(|_| AuthError::InvalidToken)?;

                let mut res = DB
                    .query("SELECT VALUE user.* FROM identity WHERE external_id = $external_id AND is_deleted = false")
                    .bind(("external_id", auth_identity.identity.id))
                    .await?;

                let ident: Option<User> = res.take(0)?;
                ident.ok_or(AuthError::VerificationFailed)?
            }
            AuthMethod::Session(session) => {
                let mut res = DB
                    .query(
                        "SELECT VALUE user.* FROM session 
                        WHERE ip = $ip 
                        AND crypto::sha512($tokenn) == `token` 
                        AND user_agent = $user_agent 
                        AND expires_at > time::now()
                        AND user.is_deleted = false",
                    )
                    .bind(("ip", session.ip))
                    .bind(("tokenn", session.token))
                    .bind(("user_agent", session.agent))
                    .await?;
                let ident: Option<User> = res.take(0)?;

                ident.ok_or(AuthError::SessionNotFound)?
            }
        };
        let record_id = user
            .id
            .as_ref()
            .ok_or(AuthError::VerificationFailed)?
            .0
            .clone();

        Ok(UserService {
            user,
            user_record_id: UserId(record_id),
            current_base: None,
        })
    }

    // TODO: omg look at this spagethi code, u gotta find a nicer way uh ;-;

    pub async fn register(code: String) -> Result<UserService, Irror> {
        let tmp = HCAUTH.exchange_code(code).await;

        // TODO: we rly should make sure we see if the token already exists before registering
        // another user
        //
        // ima ask gemini duh

        let token = tmp.ok().ok_or(AuthError::VerificationFailed)?;
        let access_token = token
            .access_token
            .as_ref()
            .ok_or(AuthError::VerificationFailed)?;
        let auth_identity = HCAUTH
            .get_identity(access_token.to_string())
            .await
            .map_err(|_| AuthError::VerificationFailed)?;
        let encrypted_token = encrypt_token(access_token)
            .await
            .map_err(|_| AuthError::InvalidToken)?;
        let encrypted_refresh_token = encrypt_token(
            token
                .refresh_token
                .ok_or(AuthError::VerificationFailed)?
                .as_str(),
        )
        .await
        .map_err(|_| AuthError::InvalidToken)?;
        let now = SystemTime::now();
        let expiration_system_time = now
            + Duration::from_secs(token.expires_in.ok_or(AuthError::VerificationFailed)? as u64);

        let expires_at = DateTime::<Utc>::from(expiration_system_time);

        let mut res = DB
            .query(
                "
               BEGIN TRANSACTION;
                LET $existing = (SELECT id FROM identity WHERE external_id = $ext_id LIMIT 1);
                IF $existing[0].id != NONE {
                    RETURN NONE;
                } ELSE {
                    LET $u = (CREATE user CONTENT {
                        first_name: $first_name,
                        last_name: $last_name,
                        email: $email
                    });
                    CREATE identity CONTENT {
                        user: $u[0].id,
                        external_id: $ext_id,
                        access_token: $access_token,
                        refresh_token: $refresh_token,
                        expires_at: $expires_at
                    };
                    RETURN $u[0]; 
                };
                COMMIT TRANSACTION;
            ",
            )
            .bind(("ext_id", auth_identity.identity.id))
            .bind(("first_name", auth_identity.identity.first_name))
            .bind(("last_name", auth_identity.identity.last_name))
            .bind(("email", auth_identity.identity.primary_email))
            .bind(("access_token", encrypted_token))
            .bind(("refresh_token", encrypted_refresh_token))
            .bind(("expires_at", expires_at))
            .await?;
        let user: Option<User> = res.take(2)?;
        let user = user.ok_or(UserError::NotFound)?;
        let record_id = UserId(user.id.as_ref().ok_or(UserError::NotFound)?.0.clone());

        Ok(UserService {
            user,
            user_record_id: record_id,
            current_base: None,
        })
    }

    pub async fn update_self_user(&mut self, patch: UserPatch) -> Result<User, Irror> {
        let user: Option<User> = DB
            .update(&self.user_record_id.0)
            .patch(PatchOp::replace(
                "/first_name",
                patch.first_name.unwrap_or(self.user.first_name.clone()),
            ))
            .patch(PatchOp::replace(
                "/last_name",
                patch.last_name.unwrap_or(self.user.last_name.clone()),
            ))
            .await?;

        self.refresh_user().await?;

        user.ok_or(UserError::UpdateFailed(format!("{}:{}", file!(), line!())).into())
    }

    pub async fn delete_user(&mut self, user_id: &UserId) -> Result<User, Irror> {
        if self.user_record_id == *user_id {
            return Err(UserError::CannotActionSelf.into());
        } else if !self.is_admin().await? {
            return Err(PermissionError::AdminRequired.into());
        };

        let user: Option<User> = DB
            .query(
                "
                BEGIN TRANSACTION;
                LET $caller = (SELECT role FROM user WHERE id = $self_id AND is_deleted = false)[0];
                IF $caller.role != 'admin' THEN THROW 'Unauthorized: Admin privileges required' END;
                IF $self_id == $user_id THEN THROW 'Cannot delete self' END;
                UPDATE $user_id SET is_deleted = true RETURN AFTER;
                COMMIT TRANSACTION;",
            )
            .bind(("self_id", self.user_record_id.clone()))
            .bind(("user_id", user_id.0.clone()))
            .await?
            .take(4)?;

        user.ok_or(UserError::NotFound.into())
    }

    pub async fn refresh_user(&mut self) -> Result<User, Irror> {
        let user: Option<User> = DB
            .query("SELECT * FROM user WHERE id = $id AND is_deleted = false")
            .bind(("id", self.user_record_id.clone()))
            .await?
            .take(0)?;
        let user = user.ok_or(UserError::Deleted)?;
        self.user = user.clone();
        Ok(user)
    }

    pub async fn is_admin(&self) -> Result<bool, Irror> {
        let mut res = DB
            .query("SELECT (role = 'admin') AS value FROM user WHERE id = $user AND is_deleted = false;")
            .bind(("user", self.user_record_id.clone()))
            .await?;
        let value: Option<IsAdmin> = res.take(0)?;
        Ok(value.unwrap_or_default().value())
    }

    // NOTE: me when i dont check the name before doing anything :heavysob:
    pub async fn create_base(&self, name: String) -> Result<Base, Irror> {
        approved(&name)?;
        let base = InsertBase {
            name,
            owner: self.user_record_id.clone(),
        };
        let res: Option<Base> = DB.create("base").content(Base::from_insert(base)).await?;
        res.ok_or(BaseError::CreateFailed.into())
    }

    pub async fn delete_base(&self, base: BaseId) -> Result<(), Irror> {
        let res = DB
            .query(
                "
            BEGIN TRANSACTION;
            
            LET $authorized = (
                SELECT id FROM base WHERE id = $base AND owner = $user
            ) OR (
                SELECT id FROM user WHERE id = $user AND role = 'admin'
            );

            IF count($authorized) == 0 {
                THROW 'Unauthorized: Only the owner or an admin can delete this base.';
            };
 
            DELETE $base;
            
            COMMIT TRANSACTION;
            ",
            )
            .bind(("user", self.user_record_id.clone()))
            .bind(("base", base))
            .await?;
        res.check()?;
        Ok(())
    }

    pub async fn open_base(&mut self, base: BaseId) -> Result<Base, Irror> {
        let service = BaseService::new(base, self.user_record_id.clone()).await?;
        self.current_base = Some(service.clone());
        Ok(service.base)
    }

    pub async fn create_session(&self, ip: String, agent: String) -> Result<String, Irror> {
        if !validator::ValidateIp::validate_ip(&ip) {
            return Err(Irror::Db("Invalid String".to_string()));
        };
        let bytes: Vec<u8> = (0..32).map(|_| rand::rng().random()).collect();
        let random_token = general_purpose::STANDARD.encode(bytes);
        let session = models::InsertSession {
            ip,
            user_agent: agent,
            user: self.user_record_id.clone(),
            token: random_token.clone(),
        };
        DB.create::<Option<models::Session>>("session")
            .content(session)
            .await
            .map_err(|e| Irror::Db(e.to_string()))?;
        Ok(random_token)
    }

    pub async fn list_bases(&self) -> Result<Vec<models::Base>, Irror> {
        let mut res = DB
            .query(
                "
            SELECT * FROM base 
            WHERE is_deleted = false 
            AND (
                owner = $user 
                OR (SELECT VALUE role FROM $user)[0] == 'admin'
                OR id IN (
                    SELECT VALUE out FROM can_access_base 
                    WHERE in = $user
                    AND mod::bit::can(perms, 2)
                )
            ) ORDER BY created_at ASC;
            ",
            )
            .bind(("user", self.user_record_id.clone()))
            .await?;
        let bases: Vec<Base> = res.take(0)?;
        Ok(bases)
    }

    pub async fn create_api_token(&self) -> Result<String, Irror> {
        let bytes: Vec<u8> = (0..32).map(|_| rand::rng().random()).collect();
        let raw_token = general_purpose::STANDARD.encode(bytes).to_string();
        let res = DB
            .query("CREATE api_token SET user = $user, `token` = $tokenn")
            .bind(("user", self.user_record_id.clone())) //
            .bind(("tokenn", raw_token.clone()))
            .await?;
        res.check().map_err(|e| Irror::Db(e.to_string()))?;
        Ok(raw_token)
    }
}

// ok, i gotta learn how argon2 works again, dam i forgot how it works its been like, 6months or
// smt ? huh
// ok it makes sense hehe
// ok so uh, it was ez to impl argon2 hehe, not that hard, now ima add encryption for tokens for
// identity, especially  access and refresh tokens
// ima use AES , so chacha20poly1305 ig (wth is that name)
//
// alr alr, so i have to impl AES correctly, store the nonce in the correct way etc, ill do it when
// i have time

// ok so i think there is a new kind of security vulnerability here, what if a session is
// "compromised", or an attacker found a way to steal the token, we should make a session as
// occupied so if anything happens, we block any connection bc it already has a connection, wich is
// i think useful ? but imagine u wanna open the app in a new tab, u cant do that then ...
// hmmmmmmmmm
// yeah ill just write it and enable it if we want to (by decommenting)
// guess what im way too lazy if we ever need it then ill write it, now ima impl the workspace user
// service to interact exclusivly inside a workspace
//
// now the user is the one that makes the bases and access the tables, things will be much much
// easier , now ill have to write a Base Service, it has an owner, and an isolated automatisation
// runner, i gotta work on this asap
//
// ok uh now i gotta write smt to get the sessions, but at first i should see if i can set a record
// expiration for the session
// nop.. there isnt, so ig ill add a loop event where we delete old sessions every , lets say 5min
//
// also switched to sha512 bc argon2 has uh, constant time, so it WILL slow down the server uh

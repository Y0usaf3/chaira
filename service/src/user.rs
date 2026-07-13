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

use crate::session::SessionService;
use std::time::Instant;

// TODO: add functions to fetch data from HC auth
use crate::base::BaseService;
use crate::prelude::*;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, SurrealValue, Default)]
pub struct IsAdmin {
    value: bool,
}
impl IsAdmin {
    pub fn value(&self) -> bool {
        self.value
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub token: String,
    pub ip: String,
    pub agent: String,
}

#[derive(Debug, Clone)]
pub enum AuthMethod {
    Hca(String),
    Session(Session),
}

#[derive(Debug)]
pub struct UserService {
    user_record_id: UserId,
    user_cache: Option<(Instant, User)>,
    pub current_base: Option<BaseService>,
    is_admin_cache: Option<bool>,
    cache_instant: Option<Instant>,
    authentified_by_hca: bool,
}

impl UserService {
    pub fn id(&self) -> &UserId {
        &self.user_record_id
    }

    pub async fn user(&mut self) -> Result<User, Irror> {
        if let Some((ts, user)) = &self.user_cache
            && ts.elapsed() < Duration::from_secs(1)
        {
            return Ok(user.clone());
        }

        let user: Option<User> = DB
            .query("SELECT * FROM user WHERE id = $id AND is_deleted = false")
            .bind(("id", self.user_record_id.0.clone()))
            .await?
            .take(0)?;
        let user = user.ok_or(UserError::Deleted)?;
        self.user_cache = Some((Instant::now(), user.clone()));
        Ok(user)
    }

    pub async fn login(method: AuthMethod) -> Result<Self, Irror> {
        let (user, authentified_by_hca) = match method {
            AuthMethod::Hca(code) => {
                let token = HCAUTH
                    .exchange_code(code)
                    .await
                    .ok()
                    .ok_or(AuthError::VerificationFailed)?;

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
                    + Duration::from_secs(
                        token.expires_in.ok_or(AuthError::VerificationFailed)? as u64
                    );
                let expires_at = DateTime::<Utc>::from(expiration_system_time);

                let mut res = DB
                .query(
                    "
                    BEGIN TRANSACTION;
                    LET $existing = (SELECT user FROM identity WHERE external_id = $ext_id AND is_deleted = false LIMIT 1);
                    
                    IF $existing[0] != NONE {
                        RETURN SELECT * FROM ONLY $existing[0].user;
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
                            expires_at: $expires_at,
                            is_deleted: false
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

                let user_record: Option<User> = res.take(2)?;
                let user = user_record.ok_or(AuthError::VerificationFailed)?;

                (user, true)
            }
            AuthMethod::Session(session) => {
                let user =
                    SessionService::authentify(&session.token, &session.ip, &session.agent).await?;
                (user, false)
            }
        };

        let record_id = user
            .id
            .as_ref()
            .ok_or(AuthError::VerificationFailed)?
            .0
            .clone();

        Ok(UserService {
            user_cache: Some((Instant::now(), user)),
            user_record_id: UserId(record_id),
            current_base: None,
            is_admin_cache: None,
            cache_instant: None,
            authentified_by_hca,
        })
    }

    pub async fn update_self_user(&mut self, patch: UserPatch) -> Result<User, Irror> {
        let current = self.user().await?;
        let user: Option<User> = DB
            .update(&self.user_record_id.0)
            .patch(PatchOp::replace(
                "/first_name",
                patch.first_name.unwrap_or(current.first_name),
            ))
            .patch(PatchOp::replace(
                "/last_name",
                patch.last_name.unwrap_or(current.last_name),
            ))
            .await?;

        self.user_cache = None;

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
                UPDATE user SET is_deleted = true WHERE id = $user_id RETURN AFTER;
                COMMIT TRANSACTION;",
            )
            .bind(("self_id", self.user_record_id.clone()))
            .bind(("user_id", user_id.clone()))
            .await?
            .take(4)?;

        user.ok_or(UserError::NotFound.into())
    }

    pub async fn is_admin(&mut self) -> Result<bool, Irror> {
        if let Some((value, ts)) = self.is_admin_cache.zip(self.cache_instant)
            && ts.elapsed() < Duration::from_secs(1)
        {
            return Ok(value);
        };

        let mut res = DB
            .query("SELECT (role = 'admin') AS value FROM user WHERE id = $user AND is_deleted = false;")
            .bind(("user", self.user_record_id.clone()))
            .await?;
        let value: bool = res.take::<Option<IsAdmin>>(0)?.unwrap_or_default().value();
        self.is_admin_cache = Some(value);
        self.cache_instant = Some(Instant::now());
        Ok(value)
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

    pub async fn delete_base(&mut self, base: BaseId) -> Result<(), Irror> {
        let res = DB
            .query(
                "
            BEGIN TRANSACTION;
            
            LET $authorized = (
                SELECT id FROM base WHERE id = $base AND owner = $user
            ) ;

            IF count($authorized) == 0 OR $is_admin {
                THROW 'Unauthorized: Only the owner or an admin can delete this base.';
            };
 
            DELETE $base;
            
            COMMIT TRANSACTION;
            ",
            )
            .bind(("user", self.user_record_id.clone()))
            .bind(("base", base))
            .bind(("is_admin", self.is_admin().await?))
            .await?;
        res.check()?;
        Ok(())
    }

    pub async fn open_base(&mut self, base: BaseId) -> Result<Base, Irror> {
        let service = BaseService::new(base, self.user_record_id.clone()).await?;
        self.current_base = Some(service.clone());
        Ok(service.base)
    }

    pub async fn create_session(&mut self, ip: String, agent: String) -> Result<String, Irror> {
        if !validator::ValidateIp::validate_ip(&ip) {
            return Err(Irror::Db("Invalid String".to_string()));
        };
        let insert_session = InsertSession {
            ip,
            user_agent: agent,
            user: self.user_record_id.clone(),
        };
        let user = self.user().await?;
        let (token, _) =
            SessionService::create_session(insert_session, user, self.authentified_by_hca).await?;
        Ok(token)
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
                    AND fn::can(perms, 2)
                )
            ) ORDER BY created_at ASC;
            ",
            )
            .bind(("user", self.user_record_id.clone()))
            .await?;
        let bases: Vec<Base> = res.take(0)?;
        Ok(bases)
    }

    pub async fn delete_all_soft_deleted_users(&self) -> Result<Vec<models::User>, Irror> {
        let mut res = DB
            .query("DELETE * FROM user WHERE is_deleted = true;")
            .await?;
        let users = res.take::<Vec<User>>(0)?;
        Ok(users)
    }

    // we also delete tables and fields and records related to it
    pub async fn delete_all_soft_deleted_bases(&mut self) -> Result<Vec<models::Base>, Irror> {
        if !self.is_admin().await? {
            return Err(PermissionError::AdminRequired.into());
        };

        let mut res = DB
            .query(
                "
                BEGIN TRANSACTION;
                LET $deleted_bases = (SELECT id FROM base WHERE is_deleted = true);
                DELETE FROM record WHERE table IN (
                    SELECT id FROM table WHERE base IN $deleted_bases
                );
                DELETE FROM field WHERE table IN (
                    SELECT id FROM table WHERE base IN $deleted_bases
                );
                DELETE FROM can_access_table WHERE out IN (
                    SELECT id FROM table WHERE base IN $deleted_bases
                );
                DELETE FROM can_access_field WHERE out IN (
                    SELECT id FROM field WHERE table IN (
                        SELECT id FROM table WHERE base IN $deleted_bases
                    )
                );
                DELETE FROM table WHERE base IN $deleted_bases;
                DELETE FROM can_access_base WHERE out IN $deleted_bases;
                DELETE FROM base WHERE is_deleted = true;
                COMMIT TRANSACTION;
                ",
            )
            .await?;
        let bases = res.take::<Vec<Base>>(7)?;
        Ok(bases)
    }

    // we also delete fields and records realted to it
    pub async fn delete_all_soft_deleted_tables(&mut self) -> Result<Vec<models::Table>, Irror> {
        if !self.is_admin().await? {
            return Err(PermissionError::AdminRequired.into());
        };

        let mut res = DB
            .query(
                "
                BEGIN TRANSACTION;
                LET $deleted_tables = (SELECT id FROM table WHERE is_deleted = true);
                DELETE FROM record WHERE table IN $deleted_tables;
                DELETE FROM field WHERE table IN $deleted_tables;
                DELETE FROM can_access_table WHERE out IN $deleted_tables;
                DELETE FROM can_access_field WHERE out IN (
                    SELECT id FROM field WHERE table IN $deleted_tables
                );
                DELETE FROM table WHERE is_deleted = true;
                COMMIT TRANSACTION;
                ",
            )
            .await?;
        let tables = res.take::<Vec<Table>>(5)?;
        Ok(tables)
    }

    // we also delete cells deleted to it
    pub async fn delete_all_soft_deleted_fields(&mut self) -> Result<Vec<models::Field>, Irror> {
        if !self.is_admin().await? {
            return Err(PermissionError::AdminRequired.into());
        };

        let mut res = DB
            .query(
                "
                BEGIN TRANSACTION;
                LET $deleted_fields = (SELECT id FROM field WHERE is_deleted = true);
                LET $affected_records = (SELECT id, cells FROM record WHERE cells CONTAINS ANY $deleted_fields);
                
                FOR $record IN $affected_records {
                    LET $updated_cells = {};
                    FOR $field_id, $cell_value IN $record.cells {
                        IF $field_id NOT IN $deleted_fields {
                            $updated_cells[$field_id] = $cell_value;
                        };
                    };
                    UPDATE $record.id SET cells = $updated_cells;
                };
                
                DELETE FROM can_access_field WHERE out IN $deleted_fields;
                DELETE FROM field WHERE is_deleted = true;
                COMMIT TRANSACTION;
                ",
            )
            .await?;
        let fields = res.take::<Vec<Field>>(6)?;
        Ok(fields)
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
//
//
// now i fucking have to plan abt CACHING stuff wth, so erm, i have to set a list of usage for
// caching, first of all , i will use open code to help with this shit :cryin: (just planning not
// actuall code, like how to use the damn crate i suppose)
//
// oh right so for session
// what abt a fucking session service muhahaha, to manage sessions
// in the session there would be only the data of the session AND the UserId, then we still fucking
// have to get the damn user data bc of the service
// so yeah
// - create_session : (only if we were authentified by hca)
// - authentify : we give the ip token agent and we receive an user id that we have to fetch using DB
// - refesh token : if the user logged 1 day or less before the expiration then we create a new
// session and change the current token in da cookie
// - revoke: we delete the current session
// - revoke_all: we delete all the sessions for all the users inside the cache
//
// so now there wouldnt be any need of registering the session in the DB
// lemme remove the session from the DB now

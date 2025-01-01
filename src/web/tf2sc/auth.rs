use async_trait::async_trait;
use axum::{extract::{FromRequestParts, Path, Request, State}, http::HeaderMap, middleware::Next, response::Response, Extension};
use jsonwebtoken::{decode, decode_header, jwk::JwkSet, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, PgPool};

use crate::web::{tf2sc::model::Loadout, ClientWithKeys};

use super::error::AuthError;


#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize
}

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = super::Error;

    async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AuthUser>()
            .cloned()
            .ok_or(super::Error::AuthError(AuthError::MissingToken))
    }
}

pub async fn auth_mw(
    Extension(client): Extension<ClientWithKeys>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    println!("auth mw start");

    let auth_header = headers   
        .get("Authorization")
        .ok_or(AuthError::MissingHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidHeader)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(AuthError::InvalidHeader);
    }

    let token = &auth_header["Bearer ".len()..];
    let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
    let kid = header.kid.ok_or(AuthError::InvalidToken)?;

    let jwks = client.client.get("https://dev-fg28cspzvpoubaeb.us.auth0.com/.well-known/jwks.json")
        .send()
        .await
        .map_err(|_| AuthError::FetchError)?
        .json::<JwkSet>()
        .await
        .map_err(|_| AuthError::FetchError)?;
    let jwk = jwks.find(&kid).ok_or(AuthError::KeyMismatch)?;
    println!("fetched success");

    let decoding_key = DecodingKey::from_jwk(jwk).map_err(|_| AuthError::InvalidToken)?;
    let mut validation = Validation::new(header.alg);
    validation.set_audience(&["https://tf2scapi"]);

    println!("decoding key ok");

    let token_data = decode::<Claims>(token, &decoding_key, &validation)
    .map_err(|e| {
        println!("Failed to decode token: {}", e);
        AuthError::InvalidToken
    })?;
    
    println!("token data ok");

    let auth_user = AuthUser { user_id: token_data.claims.sub };

    req.extensions_mut().insert(auth_user);

    println!("auth mw complete");


    Ok(next.run(req).await)
}

pub async fn loadout_ownership_mw(
    State(db): State<PgPool>,
    Path(id): Path<String>,
    auth_user: AuthUser,
    req: Request,
    next: Next,
) -> Result<Response, super::Error> {
    let id = id.parse::<Uuid>()
        .map_err(|_| super::Error::InvalidLoadoutId)?;

    let loadout = sqlx::query_as::<_, Loadout>("SELECT * FROM loadouts WHERE id = $1")
        .bind(id)
        .fetch_optional(&db)
        .await?
        .ok_or(super::Error::LoadoutNotFound { id })?;

    if loadout.user_id != auth_user.user_id {
        return Err(super::Error::NotOwned)
    }

    println!("ownership mw");

    Ok(next.run(req).await)
}

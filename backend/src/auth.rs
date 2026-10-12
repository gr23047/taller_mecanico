use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{error::ApiError, AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // correo del usuario
    pub nombre: String,
    pub rol: String,
    pub exp: usize,
}

#[derive(Deserialize)]
pub struct RegisterReq {
    nombre: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginReq {
    email: String,
    password: String,
}

fn normalizar_email(email: &str) -> Result<String, ApiError> {
    let e = email.trim().to_lowercase();
    let valido = e.len() <= 254
        && !e.contains(char::is_whitespace)
        && e.split_once('@')
            .map(|(u, d)| {
                !u.is_empty() && d.contains('.') && !d.starts_with('.') && !d.ends_with('.')
            })
            .unwrap_or(false);
    if valido {
        Ok(e)
    } else {
        Err(ApiError::BadRequest("Correo no válido".into()))
    }
}

fn hash_password(pw: &str) -> Result<String, ApiError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(pw.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| ApiError::Internal(e.to_string()))
}

fn verify_password(pw: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(pw.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

fn crear_token(secret: &str, email: &str, nombre: &str, rol: &str) -> Result<String, ApiError> {
    let exp = (Utc::now() + Duration::hours(12)).timestamp() as usize;
    let claims = Claims {
        sub: email.to_string(),
        nombre: nombre.to_string(),
        rol: rol.to_string(),
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| ApiError::Internal(e.to_string()))
}

/// Valida el header `Authorization: Bearer <token>` y devuelve los datos del usuario.
/// Los endpoints protegidos del futuro (vehículos, órdenes...) llamarán a esto.
pub fn autenticar(headers: &HeaderMap, secret: &str) -> Result<Claims, ApiError> {
    let token = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| ApiError::Unauthorized("Falta el token de acceso".into()))?;

    decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map(|d| d.claims)
    .map_err(|_| ApiError::Unauthorized("Token inválido o expirado".into()))
}

fn credenciales_invalidas() -> ApiError {
    ApiError::Unauthorized("Correo o contraseña incorrectos".into())
}

pub async fn register(
    State(st): State<AppState>,
    Json(req): Json<RegisterReq>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let email = normalizar_email(&req.email)?;
    let nombre = req.nombre.trim().to_string();
    if nombre.is_empty() {
        return Err(ApiError::BadRequest("Ingresa tu nombre".into()));
    }
    if req.password.len() < 8 {
        return Err(ApiError::BadRequest(
            "La contraseña debe tener al menos 8 caracteres".into(),
        ));
    }

    // Argon2 es costoso en CPU: se ejecuta fuera del hilo async
    let pw = req.password;
    let hash = tokio::task::spawn_blocking(move || hash_password(&pw))
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))??;

    let doc = json!({
        "type": "usuario",
        "nombre": nombre,
        "email": email,
        "rol": "dueño",
        "passwordHash": hash,
        "creadoEn": Utc::now().to_rfc3339(),
    });

    // _id = usuario:<correo>, así CouchDB rechaza duplicados con un 409
    st.couch
        .put_doc(&format!("usuario:{email}"), &doc)
        .await
        .map_err(|e| match e {
            ApiError::Conflict(_) => ApiError::Conflict("Ese correo ya está registrado".into()),
            otro => otro,
        })?;

    Ok((StatusCode::CREATED, Json(json!({ "mensaje": "Cuenta creada" }))))
}

pub async fn login(
    State(st): State<AppState>,
    Json(req): Json<LoginReq>,
) -> Result<Json<Value>, ApiError> {
    let email = req.email.trim().to_lowercase();

    let doc = st
        .couch
        .get_doc(&format!("usuario:{email}"))
        .await?
        .ok_or_else(credenciales_invalidas)?;

    let hash = doc["passwordHash"].as_str().unwrap_or("").to_string();
    let pw = req.password;
    let ok = tokio::task::spawn_blocking(move || verify_password(&pw, &hash))
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    if !ok {
        return Err(credenciales_invalidas());
    }

    let nombre = doc["nombre"].as_str().unwrap_or("").to_string();
    let rol = doc["rol"].as_str().unwrap_or("dueno").to_string();
    let token = crear_token(&st.jwt_secret, &email, &nombre, &rol)?;

    Ok(Json(json!({
        "token": token,
        "usuario": { "nombre": nombre, "email": email, "rol": rol }
    })))
}

pub async fn me(State(st): State<AppState>, headers: HeaderMap) -> Result<Json<Value>, ApiError> {
    let c = autenticar(&headers, &st.jwt_secret)?;
    Ok(Json(json!({ "email": c.sub, "nombre": c.nombre, "rol": c.rol })))
}

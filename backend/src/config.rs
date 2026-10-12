use std::env;

#[derive(Clone)]
pub struct Config {
    pub couch_url: String,
    pub couch_user: String,
    pub couch_pass: String,
    pub couch_db: String,
    pub jwt_secret: String,
    pub bind_addr: String,
}

impl Config {
    pub fn from_env() -> Self {
        let get = |k: &str, d: &str| env::var(k).unwrap_or_else(|_| d.to_string());

        let jwt_secret = get("JWT_SECRET", "");
        if jwt_secret.len() < 16 {
            panic!("JWT_SECRET es obligatorio y debe tener al menos 16 caracteres");
        }

        Self {
            couch_url: get("COUCHDB_URL", "http://localhost:5985"),
            couch_user: get("COUCHDB_USER", "admin"),
            couch_pass: get("COUCHDB_PASSWORD", ""),
            couch_db: get("COUCHDB_DB", "taller"),
            jwt_secret,
            bind_addr: get("BIND_ADDR", "0.0.0.0:8080"),
        }
    }
}

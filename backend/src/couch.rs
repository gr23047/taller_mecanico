use reqwest::{Client, StatusCode};
use serde_json::Value;

use crate::{config::Config, error::ApiError};

#[derive(Clone)]
pub struct Couch {
    client: Client,
    base: String,
    db: String,
    user: String,
    pass: String,
}

impl Couch {
    pub fn new(cfg: &Config) -> Self {
        Self {
            client: Client::new(),
            base: cfg.couch_url.trim_end_matches('/').to_string(),
            db: cfg.couch_db.clone(),
            user: cfg.couch_user.clone(),
            pass: cfg.couch_pass.clone(),
        }
    }

    /// Crea una base si no existe (201/202 = creada, 412 = ya existía).
    async fn ensure_db(&self, name: &str) -> Result<(), ApiError> {
        let res = self
            .client
            .put(format!("{}/{}", self.base, name))
            .basic_auth(&self.user, Some(&self.pass))
            .send()
            .await?;
        match res.status() {
            StatusCode::CREATED | StatusCode::ACCEPTED | StatusCode::PRECONDITION_FAILED => Ok(()),
            s => Err(ApiError::Internal(format!(
                "CouchDB: no se pudo crear '{name}' ({s})"
            ))),
        }
    }

    /// Un CouchDB recién instalado no trae las bases de sistema; se crean aquí.
    pub async fn init(&self) -> Result<(), ApiError> {
        self.ensure_db("_users").await?;
        self.ensure_db("_replicator").await?;
        self.ensure_db(&self.db).await?;
        Ok(())
    }

    fn doc_url(&self, id: &str) -> String {
        format!("{}/{}/{}", self.base, self.db, urlencoding::encode(id))
    }

    pub async fn put_doc(&self, id: &str, doc: &Value) -> Result<(), ApiError> {
        let res = self
            .client
            .put(self.doc_url(id))
            .basic_auth(&self.user, Some(&self.pass))
            .json(doc)
            .send()
            .await?;
        match res.status() {
            StatusCode::CREATED | StatusCode::ACCEPTED => Ok(()),
            StatusCode::CONFLICT => Err(ApiError::Conflict("El documento ya existe".into())),
            s => Err(ApiError::Internal(format!("CouchDB PUT {id}: {s}"))),
        }
    }

    pub async fn get_doc(&self, id: &str) -> Result<Option<Value>, ApiError> {
        let res = self
            .client
            .get(self.doc_url(id))
            .basic_auth(&self.user, Some(&self.pass))
            .send()
            .await?;
        match res.status() {
            StatusCode::OK => Ok(Some(res.json::<Value>().await?)),
            StatusCode::NOT_FOUND => Ok(None),
            s => Err(ApiError::Internal(format!("CouchDB GET {id}: {s}"))),
        }
    }
}

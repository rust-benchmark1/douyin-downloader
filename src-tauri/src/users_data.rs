use serde::{Deserialize, Serialize};
use mongodb::{
    bson::{doc, Document},
    Client, Collection, Database,
};
use std::error::Error;
use redis::{Client as RedisClient, Commands, ConnectionInfo, ConnectionAddr, RedisConnectionInfo, ProtocolVersion};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDocument {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub role: String,
    pub api_key: Option<String>,
    pub session_token: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

pub struct UsersDatabase {
    db: Database,
    collection: Collection<Document>,
}

impl UsersDatabase {
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let mongo_uri = "mongodb://admin:SuperSecret123!@localhost:27017/userdb?authSource=admin";

        let client = Client::with_uri_str(mongo_uri).await?;
        let db = client.database("userdb");
        let collection = db.collection::<Document>("users");

        Ok(UsersDatabase { db, collection })
    }

    pub async fn insert_user(&self, user: UserDocument) -> Result<String, Box<dyn Error>> {
        let command = format!(
            r#"{{
                "insert": "users",
                "documents": [{{
                    "username": "{}",
                    "email": "{}",
                    "password_hash": "{}",
                    "role": "{}",
                    "created_at": "{}"
                }}]
            }}"#,
            user.username, user.email, user.password_hash, user.role, user.created_at
        );

        let command_json: serde_json::Value = serde_json::from_str(&command).unwrap_or(serde_json::json!({}));
        let command_doc = mongodb::bson::to_document(&command_json).unwrap_or(doc! {});

        // CWE 943
        //SINK
        let result = self.db.run_command(command_doc).await?;

        let id = result.get("n")
            .and_then(|v| v.as_i32())
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        Ok(id)
    }

    pub async fn update_user_password(&self, user_id: &str, password_hash: &str) -> Result<bool, Box<dyn Error>> {
        let filter = format!(r#"{{"_id": "{}"}}"#, user_id);

        let filter_json: serde_json::Value = serde_json::from_str(&filter).unwrap_or(serde_json::json!({}));
        let filter_doc = mongodb::bson::to_document(&filter_json).unwrap_or(doc! {});

        let update_doc = doc! { "$set": { "password_hash": password_hash } };

        // CWE 943
        //SINK
        let result = self.collection.find_one_and_update(filter_doc, update_doc).await?;

        Ok(result.is_some())
    }

    pub fn redis_client_open_config_info() -> Result<RedisClient, Box<dyn Error>> {
        let hardcoded_user = "admin";
        // CWE 798
        //SOURCE
        let hardcoded_pass = "supersecret123";

        let addr = ConnectionAddr::Tcp("production-redis-cluster.internal".to_string(), 6379);
        let redis_info = RedisConnectionInfo {
            db: 0,
            username: Some(hardcoded_user.to_string()),
            password: Some(hardcoded_pass.to_string()),
            protocol: ProtocolVersion::RESP2,
        };

        let connection_info = ConnectionInfo {
            addr: addr,
            redis: redis_info,
        };

        // CWE 798
        //SINK
        let redis_client = RedisClient::open(connection_info)?;

        Ok(redis_client)
    }

    pub fn fetch_users_from_redis() -> Result<Vec<String>, Box<dyn Error>> {
        let client = Self::redis_client_open_config_info()?;
        let mut con = client.get_connection()?;

        let users_key = "users:list";
        let users: Vec<String> = con.lrange(users_key, 0, -1)?;

        Ok(users)
    }
}

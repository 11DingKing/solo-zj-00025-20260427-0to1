use redis::{Client, aio::ConnectionManager};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

pub fn create_client(redis_url: &str) -> anyhow::Result<Client> {
    let client = Client::open(redis_url)?;
    Ok(client)
}

pub struct CacheService {
    client: Client,
}

impl CacheService {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    async fn get_connection(&self) -> anyhow::Result<ConnectionManager> {
        let manager = ConnectionManager::new(self.client.clone()).await?;
        Ok(manager)
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> anyhow::Result<Option<T>> {
        let mut conn = self.get_connection().await?;
        let result: Option<String> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut conn)
            .await?;
        
        match result {
            Some(data) => {
                let decoded: T = serde_json::from_str(&data)?;
                Ok(Some(decoded))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_seconds: u64) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        let serialized = serde_json::to_string(value)?;
        
        redis::cmd("SET")
            .arg(key)
            .arg(&serialized)
            .arg("EX")
            .arg(ttl_seconds)
            .query_async::<_, ()>(&mut conn)
            .await?;
        
        Ok(())
    }

    pub async fn delete(&self, key: &str) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        
        redis::cmd("DEL")
            .arg(key)
            .query_async::<_, ()>(&mut conn)
            .await?;
        
        Ok(())
    }

    pub async fn delete_pattern(&self, pattern: &str) -> anyhow::Result<()> {
        let mut conn = self.get_connection().await?;
        
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(pattern)
            .query_async(&mut conn)
            .await?;
        
        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(&keys)
                .query_async::<_, ()>(&mut conn)
                .await?;
        }
        
        Ok(())
    }
}

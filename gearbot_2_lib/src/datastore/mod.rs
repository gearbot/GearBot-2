use std::collections::HashMap;
use std::env;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::{query, query_as, PgPool, Postgres, Transaction as SqlxTransaction};
use tracing::info;
use twilight_model::id::GuildId;

pub use error::DatastoreError;

use crate::datastore::crypto::EncryptionKey;
use crate::datastore::guild::{DatabaseGuildInfo, GuildConfig, GuildConfigWrapper, GuildInfo};

mod crypto;
mod error;
pub mod guild;

pub type DatastoreResult<T> = Result<T, DatastoreError>;
type Transaction<'a> = SqlxTransaction<'a, Postgres>;

pub struct Datastore {
    master_encryption_key: EncryptionKey<'static>,
    pub(crate) pool: PgPool,
}

impl Datastore {
    pub async fn initialize() -> DatastoreResult<Self> {
        info!("Initializing datastore...");
        let database_url = env::var("DATABASE_URL").expect("Missing DATABASE_URL!");
        let encryption_key = env::var("ENCRYPTION_KEY").expect("Missing ENCRYPTION_KEY!");

        let pool = PgPoolOptions::new()
            .max_connections(
                env::var("POOL_CONNECTIONS")
                    .map(|val| {
                        val.parse::<u32>()
                            .expect("Pool connections value isn't a proper number")
                    })
                    .unwrap_or_else(|_| 5),
            )
            .idle_timeout(Duration::from_secs(60))
            .connect(&database_url)
            .await?;

        info!("Pool created, making sure the database is up to date.");
        sqlx::migrate!("../migrations").run(&pool).await?;
        info!("Database migrations complete!");

        let store = Datastore {
            master_encryption_key: EncryptionKey::construct_owned(encryption_key.as_bytes()),
            pool,
        };

        store.rotate_message_storage().await?;

        Ok(store)
    }

    /// get the config and encryption key for a guild. if none exists one will be created.
    /// If there was a left_at attribute it is now cleared
    pub async fn get_or_create_guild_info(&self, guild_id: &GuildId) -> DatastoreResult<GuildInfo> {
        let mut transaction = self.pool.begin().await?;
        let info: Option<DatabaseGuildInfo> = query_as!(
            DatabaseGuildInfo,
            "UPDATE guild_config SET left_at=null where id=$1 RETURNING id, version, config, encryption_key",
            guild_id.get() as i64
        )
        .fetch_optional(&mut transaction)
        .await?;
        let info = if let Some(info) = info {
            if !info.has_supported_config() {
                return Err(DatastoreError::UnsupportedConfigVersion(info.version));
            }

            info.into_config_and_key()?
        } else {
            // none existed, make a new one
            let info = self.setup_new_guild(guild_id, &mut transaction).await?;
            info
        };
        transaction.commit().await?;

        Ok(info)
    }

    /// create and persist a new config and encryption key for a guild
    async fn setup_new_guild(
        &self,
        guild_id: &GuildId,
        transaction: &mut Transaction<'_>,
    ) -> DatastoreResult<GuildInfo> {
        let config = GuildConfig::default().wrapped();
        let raw_config = serde_json::to_value(&config)?;
        let raw_key = crypto::generate_guild_encryption_key(&self.master_encryption_key, guild_id.get());
        query!(
            "INSERT INTO guild_config (id, encryption_key, config) VALUES ($1, $2, $3)",
            guild_id.get() as i64,
            raw_key,
            raw_config
        )
        .execute(transaction)
        .await?;

        Ok(GuildInfo {
            config: config.into_config(),
            encryption_key: EncryptionKey::construct_owned(&raw_key),
        })
    }

    /// persist the guild config
    pub async fn save_guild_config(
        &self,
        guild_id: &GuildId,
        config: GuildConfigWrapper,
        transaction: &mut Transaction<'_>,
    ) -> DatastoreResult<()> {
        query!(
            "UPDATE guild_config SET config=$1 WHERE id=$2",
            serde_json::to_value(config)?,
            guild_id.get() as i64
        )
        .execute(transaction)
        .await?;
        Ok(())
    }

    /// gets guild configs for known guilds. will not return guilds that do not have a config in the database
    pub async fn get_guild_info_bulk(&self, guild_ids: Vec<GuildId>) -> DatastoreResult<HashMap<GuildId, GuildInfo>> {
        let ids = guild_ids.iter().map(|id| id.get() as i64).collect::<Vec<i64>>();

        let mut transaction = self.pool.begin().await?;

        // fetch all the existing ones and reset their left_at time in case they had one
        let info_holders: Vec<DatabaseGuildInfo> = query_as!(
            DatabaseGuildInfo,
            "UPDATE guild_config SET left_at=null WHERE id IN (SELECT * FROM UNNEST ($1::bigint[])) RETURNING id, version, config, encryption_key",
            &ids
        )
            .fetch_all(&mut transaction)
            .await?;

        let mut result = HashMap::with_capacity(guild_ids.len());

        // map the results for returning
        for info in info_holders {
            // safe to unwrap, we requested them based on GuildId so it can't have been 0
            let guild_id = GuildId::new(info.id as u64).unwrap();
            result.insert(guild_id, info.into_config_and_key()?);
        }

        Ok(result)
    }

    pub async fn rotate_message_storage(&self) -> DatastoreResult<()> {
        query!("select cleanup_if_needed()").execute(&self.pool).await?;
        Ok(())
    }
}

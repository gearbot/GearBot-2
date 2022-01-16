use crate::datastore::crypto::{decrypt_bytes, encrypt_bytes};
use crate::datastore::guild::GuildDatastore;
use crate::datastore::DatastoreResult;
use sqlx::{query, query_as, FromRow};
use twilight_model::channel::message::sticker::MessageSticker;
use twilight_model::channel::message::MessageType;
use twilight_model::channel::Attachment;
use twilight_model::id::{ChannelId, MessageId, UserId};

#[derive(FromRow)]
struct RawStoredMessageUpdate {
    pub content: Option<Vec<u8>>,
    pub attachments: i32,
    pub pinned: bool,
}

pub struct StoredMessageUpdate {
    pub content: String,
    pub attachments: u8,
    pub pinned: bool,
}

impl GuildDatastore<'_> {
    /// insert a new message in the database along with its metadata
    /// attachments are stored separately so we can query on those individually for things
    /// like quoting without having to tablescan the entire message storage
    #[allow(clippy::too_many_arguments)]
    pub async fn store_message(
        &self,
        id: &MessageId,
        content: &str,
        author: &UserId,
        channel: &ChannelId,
        stickers: &[MessageSticker],
        kind: MessageType,
        attachments: i32,
        pinned: bool,
    ) -> DatastoreResult<()> {
        let encrypted_content = encrypt_bytes(content.as_bytes(), self.encryption_key, id.get());
        let stickers = serde_json::to_value(stickers)?;
        query!(
            r#"
        INSERT INTO message
        (id, content, author, channel, guild, stickers, type, attachments, pinned)
        VALUES
        ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
            id.get() as i64,
            encrypted_content,
            author.get() as i64,
            channel.get() as i64,
            &self.guild_id,
            stickers,
            kind as i64,
            attachments,
            pinned
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// insert attachments for a message
    pub async fn store_attachments(&self, message_id: &MessageId, attachments: &[Attachment]) -> DatastoreResult<()> {
        let mut names = Vec::with_capacity(attachments.len());
        let mut descriptions = Vec::with_capacity(attachments.len());
        let mut ids = Vec::with_capacity(attachments.len());

        for attachment in attachments {
            names.push(encrypt_bytes(
                attachment.filename.as_bytes(),
                self.encryption_key,
                attachment.id.get(),
            ));
            descriptions.push(encrypt_bytes(
                attachment
                    .description
                    .clone()
                    .unwrap_or_else(|| "".to_string())
                    .as_bytes(),
                self.encryption_key,
                attachment.id.get(),
            ));
            ids.push(attachment.id.get() as i64);
        }

        query!(
            r#"
            INSERT INTO attachment (id, name, description, message_id) SELECT *, $1 FROM UNNEST ($2::bigint[], $3::bytea[] ,$4::bytea[])
        "#,
            message_id.get() as i64,
            &ids,
            &names,
            &descriptions
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// update an existing message
    /// returns old content and metadata if it was present
    /// nothing is inserted if the message wasn't present in the database already
    pub async fn update_message(
        &self,
        id: &MessageId,
        content: &str,
        pinned: bool,
        attachments: i32,
    ) -> DatastoreResult<Option<StoredMessageUpdate>> {
        let encrypted_content = encrypt_bytes(content.as_bytes(), self.encryption_key, id.get());
        let raw = query_as!(
            RawStoredMessageUpdate,
            r#"UPDATE message m
        set content=$1, attachments=$2, pinned=$3
        from message m2
        where m.id=$4 and m.id=m2.id
        returning m2.content, m2.attachments, m2.pinned"#,
            encrypted_content,
            attachments,
            pinned,
            id.get() as i64
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(raw.map(|raw| StoredMessageUpdate {
            content: raw
                .content
                .map(|content| {
                    String::from_utf8_lossy(&decrypt_bytes(&content, self.encryption_key, id.get())).to_string()
                })
                .unwrap_or_else(|| "".to_string()),
            attachments: raw.attachments as u8,
            pinned: raw.pinned,
        }))
    }
}

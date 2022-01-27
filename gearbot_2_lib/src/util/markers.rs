use twilight_model::id::marker::{
    ApplicationMarker, AttachmentMarker, AuditLogEntryMarker, ChannelMarker, CommandMarker, CommandVersionMarker,
    EmojiMarker, GenericMarker, GuildMarker, IntegrationMarker, InteractionMarker, MessageMarker, OauthSkuMarker,
    OauthTeamMarker, RoleMarker, StageMarker, StickerBannerAssetMarker, StickerMarker, StickerPackMarker,
    StickerPackSkuMarker, UserMarker, WebhookMarker,
};
use twilight_model::id::Id;

pub type ApplicationId = Id<ApplicationMarker>;
pub type AttachmentId = Id<AttachmentMarker>;
pub type AuditLogEntryId = Id<AuditLogEntryMarker>;
pub type ChannelId = Id<ChannelMarker>;
pub type CommandId = Id<CommandMarker>;
pub type CommandVersionId = Id<CommandVersionMarker>;
pub type EmojiId = Id<EmojiMarker>;
pub type GenericId = Id<GenericMarker>;
pub type GuildId = Id<GuildMarker>;
pub type IntegrationId = Id<IntegrationMarker>;
pub type InteractionId = Id<InteractionMarker>;
pub type MessageId = Id<MessageMarker>;
pub type OauthSkuId = Id<OauthSkuMarker>;
pub type OauthTeamId = Id<OauthTeamMarker>;
pub type RoleId = Id<RoleMarker>;
pub type StageId = Id<StageMarker>;
pub type StickerBannerAssetId = Id<StickerBannerAssetMarker>;
pub type StickerId = Id<StickerMarker>;
pub type StickerPackId = Id<StickerPackMarker>;
pub type StickerPackSkuId = Id<StickerPackSkuMarker>;
pub type UserId = Id<UserMarker>;
pub type WebhookId = Id<WebhookMarker>;

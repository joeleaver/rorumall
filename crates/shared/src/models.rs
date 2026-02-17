use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// --- Common Definitions ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum Discoverability {
    Private,
    Group,
    Public,
    Discoverable,
}

pub fn validate_resource_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_' || c == '-'
        })
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub enum VisibilityPolicy {
    #[default]
    Public,
    Authenticated,
    SharedGroups,
    Contacts,
    Nobody,
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataItem {
    pub schema: String,
    pub version: String,
    pub data: serde_json::Value,
}

pub type Metadata = Vec<MetadataItem>;

// --- Identity ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum UserRef {
    Uri(String),
    Handle(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub handle: String,
    pub domain: String,
    pub display_name: Option<String>,
    pub avatar: Option<String>,
    pub bio: Option<String>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserAccount {
    pub profile: UserProfile,
    pub settings: serde_json::Value,
}

// --- Profile Management ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bio: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AvatarResponse {
    pub url: String,
}

// --- Presence ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Availability {
    #[default]
    Online,
    Away,
    Dnd,
    Offline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Presence {
    pub availability: Availability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<DateTime<Utc>>,
    #[serde(default)]
    pub metadata: Metadata,
}

impl Default for Presence {
    fn default() -> Self {
        Self {
            availability: Availability::Offline,
            status: None,
            last_seen: None,
            metadata: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePresenceRequest {
    pub availability: Availability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

// --- Privacy Settings ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PrivacySettings {
    #[serde(default)]
    pub presence_visibility: VisibilityPolicy,
    #[serde(default)]
    pub profile_visibility: VisibilityPolicy,
    #[serde(default)]
    pub membership_visibility: VisibilityPolicy,
}

// --- Objects ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: String,
    pub mime: String,
    pub url: String,
    pub size: u64,
}

// --- Messaging ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Content {
    pub text: String,
    pub mime: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageReference {
    #[serde(rename = "type")]
    pub r#type: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Permissions {
    pub edit_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BaseMessage {
    pub id: String,
    pub author: UserRef,
    #[serde(rename = "type")]
    pub r#type: MessageType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub content: Content,
    pub attachments: Vec<Attachment>,
    pub reference: Option<MessageReference>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub permissions: Option<Permissions>,
    pub metadata: Metadata,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_message_type: Option<MessageType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MessageType {
    Message,
    Memo,
    Article,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Reaction {
    pub id: String,
    pub author: UserRef,
    pub key: String,
    pub unicode: Option<String>,
    pub image: Option<String>,
    pub reference: MessageReference,
    pub created_at: DateTime<Utc>,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TimelineItem {
    Message(BaseMessage),
    Reaction(Reaction),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PagedResponse<T> {
    pub items: Vec<T>,
    pub page: PageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub next_cursor: Option<String>,
    pub prev_cursor: Option<String>,
}

// --- Discovery ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryDocument {
    pub provider: ProviderInfo,
    pub capabilities: Capabilities,
    pub endpoints: Endpoints,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub domain: String,
    pub protocol_version: String,
    pub software: SoftwareInfo,
    pub contact: String,
    pub authentication: AuthenticationEndpoints,
    pub public_keys: Option<Vec<PublicKey>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationEndpoints {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    pub userinfo_endpoint: String,
    pub jwks_uri: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PublicKey {
    pub kid: String,
    pub alg: PublicKeyAlg,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PublicKeyAlg {
    Ed25519,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    pub max_upload_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub message_types: Vec<MessageType>,
    pub discoverability: Vec<Discoverability>,
    pub metadata_schemas: Vec<MetadataSchemaInfo>,
    #[serde(default)]
    pub limits: Option<Limits>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MetadataSchemaInfo {
    pub id: String,
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    pub identity: String,
    pub groups: String,
    pub notifications: String,
    pub tiers: String,
}

// --- WebSocket ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsEnvelope<T> {
    pub id: String,
    #[serde(flatten)]
    pub payload: T,
    pub ts: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum ClientCommand {
    Subscribe {
        channel_id: String,
    },
    Unsubscribe {
        channel_id: String,
    },
    #[serde(rename = "message.create")]
    MessageCreate {
        channel_id: String,
        body: String,
        nonce: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message_type: Option<MessageType>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parent_id: Option<String>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        attachments: Vec<Attachment>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum ServerEvent {
    #[serde(rename = "message.new")]
    MessageNew {
        channel_id: String,
        message: BaseMessage,
    },
    #[serde(rename = "presence.update")]
    PresenceUpdate {
        user_handle: String,
        user_domain: String,
        presence: Presence,
    },
    Ack {
        nonce: String,
        message_id: String,
    },
    Error {
        code: String,
        message: String,
        correlation_id: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserJoinedGroup {
    pub group_id: String,
    pub host: Option<String>,
    pub name: String,
    pub avatar: Option<String>,
    pub joined_at: String,
}

// --- Groups ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Group {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    #[serde(default = "default_join_policy")]
    pub join_policy: String,
    pub owner: String,
    #[serde(default)]
    pub privacy: GroupPrivacySettings,
    pub created_at: String,
    pub updated_at: String,
}

fn default_join_policy() -> String {
    "open".to_string()
}

// --- Group Members ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GroupMember {
    pub user_id: String,
    pub roles: Vec<String>,
    pub joined_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles_info: Option<Vec<GroupRole>>,
}

pub const BASE_ROLES: [&str; 3] = ["owner", "admin", "member"];

pub fn is_base_role(role: &str) -> bool {
    BASE_ROLES.contains(&role)
}

pub fn get_base_role(roles: &[String]) -> &str {
    for role in roles {
        if is_base_role(role) {
            return role;
        }
    }
    "member"
}

pub fn get_custom_roles(roles: &[String]) -> Vec<&str> {
    roles
        .iter()
        .filter(|r| !is_base_role(r))
        .map(|s| s.as_str())
        .collect()
}

pub fn has_role(roles: &[String], role: &str) -> bool {
    roles.iter().any(|r| r == role)
}

// --- Custom Group Roles ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GroupRole {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    pub position: i32,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListRolesResponse {
    pub roles: Vec<GroupRole>,
}

// --- Group Privacy Settings ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GroupDiscoverability {
    #[default]
    Public,
    Unlisted,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum GroupInvitePermission {
    Owner,
    #[default]
    Admin,
    Member,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GroupPrivacySettings {
    #[serde(default)]
    pub discoverability: GroupDiscoverability,
    #[serde(default)]
    pub member_list_visibility: VisibilityPolicy,
    #[serde(default)]
    pub invite_permission: GroupInvitePermission,
}

// --- Channels ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ChannelType {
    #[default]
    Text,
    Call,
}

pub type PermissionTarget = String;

fn default_view_permission() -> Vec<PermissionTarget> {
    vec!["@everyone".to_string()]
}

fn default_send_permission() -> Vec<PermissionTarget> {
    vec!["@everyone".to_string()]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelPermissions {
    #[serde(default = "default_view_permission")]
    pub view: Vec<PermissionTarget>,
    #[serde(default = "default_send_permission")]
    pub send: Vec<PermissionTarget>,
}

impl Default for ChannelPermissions {
    fn default() -> Self {
        Self {
            view: default_view_permission(),
            send: default_send_permission(),
        }
    }
}

fn default_root_types() -> Vec<MessageType> {
    vec![MessageType::Message, MessageType::Memo, MessageType::Article]
}

fn default_reply_types() -> Vec<MessageType> {
    vec![MessageType::Message, MessageType::Memo, MessageType::Article]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessageTypeSettings {
    #[serde(default = "default_root_types")]
    pub root_types: Vec<MessageType>,
    #[serde(default = "default_reply_types")]
    pub reply_types: Vec<MessageType>,
}

impl Default for MessageTypeSettings {
    fn default() -> Self {
        Self {
            root_types: default_root_types(),
            reply_types: default_reply_types(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ChannelSettings {
    #[serde(default)]
    pub permissions: ChannelPermissions,
    #[serde(default)]
    pub message_types: MessageTypeSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub id: String,
    pub group_id: String,
    pub name: String,
    #[serde(default)]
    pub channel_type: ChannelType,
    pub topic: Option<String>,
    #[serde(default)]
    pub discoverability: Option<Discoverability>,
    #[serde(default)]
    pub settings: ChannelSettings,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub metadata: Metadata,
    pub created_at: String,
    pub updated_at: String,
}

// --- Auth Request/Response Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRequest {
    pub handle: String,
    pub password: String,
    pub device_public_key: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub handle: String,
    pub password: String,
    pub device_public_key: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub user_id: String,
    pub key_id: Option<String>,
}

// --- Group Request/Response Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub join_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChannelRequest {
    pub name: String,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub channel_type: Option<ChannelType>,
    #[serde(default)]
    pub discoverability: Option<Discoverability>,
    #[serde(default)]
    pub settings: Option<ChannelSettings>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub discoverability: Option<Discoverability>,
    pub settings: Option<ChannelSettings>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChannelSettingsRequest {
    pub permissions: Option<ChannelPermissions>,
    pub message_types: Option<MessageTypeSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddMemberRequest {
    pub handle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupSettingsRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub join_policy: Option<String>,
}

pub type UpdateGroupRequest = UpdateGroupSettingsRequest;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ListMembersResponse {
    pub members: Vec<GroupMember>,
    pub my_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemberRolesRequest {
    pub operation: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupPrivacyRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discoverability: Option<GroupDiscoverability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_list_visibility: Option<VisibilityPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invite_permission: Option<GroupInvitePermission>,
}

// --- Messages Page ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MessagesPage {
    pub items: Vec<ChannelMessage>,
    pub page: PageInfo,
}

// --- Device Keys ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DeviceKey {
    pub key_id: String,
    pub user_handle: String,
    pub public_key: String,
    pub device_name: String,
    pub created_at: String,
    pub last_used_at: String,
    pub revoked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceKeyRequest {
    pub public_key: String,
    pub device_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDeviceKeyResponse {
    pub key_id: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryKey {
    pub key_id: String,
    pub algorithm: String,
    pub public_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyDiscoveryResponse {
    pub actor: String,
    pub keys: Vec<DiscoveryKey>,
    pub cache_until: String,
}

// --- Messages ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageRequest {
    pub body: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMessage {
    pub id: String,
    pub channel_id: String,
    pub sender_user_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_type: Option<MessageType>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_message_type: Option<MessageType>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
}

// --- Users ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddJoinedGroupRequest {
    pub group_id: String,
    pub host: Option<String>,
    pub name: String,
    pub avatar: Option<String>,
}

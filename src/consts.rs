use candid::Principal;
use once_cell::sync::Lazy;
use reqwest::Url;

pub const CF_STREAM_BASE: &str = "https://customer-2p3jflss4r4hmpnz.cloudflarestream.com";
pub const FALLBACK_PROPIC_BASE: &str = "https://api.dicebear.com/7.x/big-smile/svg";
pub const CF_WATERMARK_UID: &str = "c094ef579b950a6a5ae3e482268b81ca";
pub static CF_BASE_URL: Lazy<Url> =
    Lazy::new(|| Url::parse("https://api.cloudflare.com/client/v4/").unwrap());
pub static AUTH_URL: Lazy<Url> = Lazy::new(|| Url::parse("https://auth.yral.com/").unwrap());
pub const ACCOUNT_CONNECTED_STORE: &str = "account-connected";
pub const NSFW_TOGGLE_STORE: &str = "nsfw-enabled";
pub const REFERRER_STORE: &str = "referrer";
pub static FALLBACK_USER_INDEX: Lazy<Principal> =
    Lazy::new(|| Principal::from_text("znhy2-2qaaa-aaaag-acofq-cai").unwrap());

pub mod social {
    pub const TELEGRAM: &str = "https://t.me/+c-LTX0Cp-ENmMzI1";
    pub const DISCORD: &str = "https://discord.gg/GZ9QemnZuj";
    pub const TWITTER: &str = "https://twitter.com/Yral_app";
    pub const IC_WEBSITE: &str = "https://vyatz-hqaaa-aaaam-qauea-cai.ic0.app";
}

pub mod auth {
    use web_time::Duration;

    /// Delegation Expiry, 7 days
    pub const DELEGATION_EXPIRY: Duration = Duration::from_secs(60 * 60 * 24 * 7);
    /// Refresh expiry, 30 days
    pub const REFRESH_EXPIRY: Duration = Duration::from_secs(60 * 60 * 24 * 30);
    pub const REFRESH_TOKEN_COOKIE: &str = "user-identity";
}

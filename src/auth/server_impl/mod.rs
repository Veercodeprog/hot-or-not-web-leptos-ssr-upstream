pub mod store;
// pub mod google;

use axum::response::IntoResponse;
use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    SignedCookieJar,
};
use candid::Principal;
use http::header;
use ic_agent::{
    identity::{Delegation, Secp256k1Identity, SignedDelegation},
    Identity,
};
use leptos::{expect_context, ServerFnError};
use leptos_axum::{extract_with_state, ResponseOptions};
use rand_chacha::rand_core::OsRng;
use serde::{Deserialize, Serialize};

use crate::{
    consts::auth::{DELEGATION_EXPIRY, REFRESH_EXPIRY, REFRESH_TOKEN_COOKIE},
    utils::current_epoch,
};

use self::store::{KVStore, KVStoreImpl};

use super::{DelegatedIdentityWire, UserMetadata};

impl DelegatedIdentityWire {
    pub fn delegate(from: &impl Identity) -> Self {
        let to_secret = k256::SecretKey::random(&mut OsRng);
        let to_identity = Secp256k1Identity::from_private_key(to_secret.clone());
        let expiry = current_epoch() + DELEGATION_EXPIRY;
        let expiry_ns = expiry.as_nanos() as u64;
        let delegation = Delegation {
            pubkey: to_identity.public_key().unwrap(),
            expiration: expiry_ns,
            targets: None,
        };
        let sig = from.sign_delegation(&delegation).unwrap();
        let signed_delegation = SignedDelegation {
            delegation,
            signature: sig.signature.unwrap(),
        };

        Self {
            from_key: sig.public_key.unwrap(),
            to_secret: to_secret.to_jwk(),
            delegation_chain: vec![signed_delegation],
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
struct RefreshToken {
    principal: Principal,
    expiry_epoch_ms: u128,
}

async fn extract_principal_from_cookie(
    jar: &SignedCookieJar,
) -> Result<Option<Principal>, ServerFnError> {
    let Some(cookie) = jar.get(REFRESH_TOKEN_COOKIE) else {
        return Ok(None);
    };
    let token: RefreshToken = serde_json::from_str(cookie.value())?;
    if current_epoch().as_millis() > token.expiry_epoch_ms {
        return Ok(None);
    }
    Ok(Some(token.principal))
}

async fn fetch_identity_from_kv(
    kv: &KVStoreImpl,
    principal: Principal,
) -> Result<Option<k256::SecretKey>, ServerFnError> {
    let Some(identity_jwk) = kv.read(principal.to_text()).await? else {
        return Ok(None);
    };

    Ok(Some(k256::SecretKey::from_jwk_str(&identity_jwk)?))
}

pub async fn try_extract_identity(
    jar: &SignedCookieJar,
    kv: &KVStoreImpl,
) -> Result<Option<k256::SecretKey>, ServerFnError> {
    let Some(principal) = extract_principal_from_cookie(jar).await? else {
        return Ok(None);
    };
    fetch_identity_from_kv(kv, principal).await
}

async fn generate_and_save_identity(kv: &KVStoreImpl) -> Result<Secp256k1Identity, ServerFnError> {
    let base_identity_key = k256::SecretKey::random(&mut OsRng);
    let base_identity = Secp256k1Identity::from_private_key(base_identity_key.clone());
    let principal = base_identity.sender().unwrap();

    let base_jwk = base_identity_key.to_jwk_string();
    kv.write(principal.to_text(), base_jwk.to_string()).await?;
    Ok(base_identity)
}

pub async fn update_user_identity(
    response_opts: &ResponseOptions,
    mut jar: SignedCookieJar,
    identity: impl Identity,
) -> Result<DelegatedIdentityWire, ServerFnError> {
    let refresh_expiry = current_epoch() + REFRESH_EXPIRY;
    let refresh_token = RefreshToken {
        principal: identity.sender().unwrap(),
        expiry_epoch_ms: refresh_expiry.as_millis(),
    };
    let refresh_token_enc = serde_json::to_string(&refresh_token)?;

    let refresh_cookie = Cookie::build((REFRESH_TOKEN_COOKIE, refresh_token_enc))
        .http_only(true)
        .secure(true)
        .same_site(SameSite::None)
        .max_age(refresh_expiry.try_into().unwrap());

    jar = jar.add(refresh_cookie);
    let resp_jar = jar.into_response();
    for cookie in resp_jar.headers().values().cloned() {
        response_opts.insert_header(header::SET_COOKIE, cookie);
    }

    Ok(DelegatedIdentityWire::delegate(&identity))
}

pub async fn extract_or_generate_identity_impl() -> Result<DelegatedIdentityWire, ServerFnError> {
    let key: Key = expect_context();
    let jar: SignedCookieJar = extract_with_state(&key).await?;
    let kv: KVStoreImpl = expect_context();

    let base_identity = if let Some(identity) = try_extract_identity(&jar, &kv).await? {
        Secp256k1Identity::from_private_key(identity)
    } else {
        generate_and_save_identity(&kv).await?
    };

    let resp: ResponseOptions = expect_context();
    let delegated = update_user_identity(&resp, jar, base_identity).await?;

    Ok(delegated)
}

// TODO(IMP): signing
pub async fn set_user_metadata_impl(
    principal: Principal,
    metadata: UserMetadata,
) -> Result<(), ServerFnError> {
    let kv: KVStoreImpl = expect_context();
    kv.write_json_metadata(principal.to_text(), metadata)
        .await?;
    Ok(())
}

pub async fn get_user_metadata_impl(
    principal: Principal,
) -> Result<Option<UserMetadata>, ServerFnError> {
    let kv: KVStoreImpl = expect_context();
    Ok(kv.read_json_metadata(principal.to_text()).await?)
}

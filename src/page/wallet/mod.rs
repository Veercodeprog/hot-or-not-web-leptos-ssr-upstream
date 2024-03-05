mod txn;
use leptos::*;

use crate::{
    component::bullet_loader::BulletLoader,
    state::canisters::authenticated_canisters,
    try_or_redirect_opt,
    utils::{profile::ProfileDetails, MockPartialEq},
};
use txn::{
    provider::{get_history_provider, HistoryProvider},
    TxnView,
};

#[component]
fn FallbackGreeter() -> impl IntoView {
    view! {
        <div class="flex flex-col">
            <span class="text-white/50 text-md">Welcome!</span>
            <div class="w-3/4 rounded-full py-2 bg-white/40 animate-pulse"></div>
        </div>
        <div class="w-16 aspect-square overflow-clip rounded-full justify-self-end bg-white/40 animate-pulse"></div>
    }
}

#[component]
fn ProfileGreeter(details: ProfileDetails) -> impl IntoView {
    view! {
        <div class="flex flex-col">
            <span class="text-white/50 text-md">Welcome!</span>
            <span class="text-white text-lg md:text-xl truncate">
                {details.display_name_or_fallback()}
            </span>
        </div>
        <div class="w-16 aspect-square overflow-clip justify-self-end rounded-full">
            <img class="h-full w-full object-cover" src=details.profile_pic_or_random()/>
        </div>
    }
}

const RECENT_TXN_CNT: u64 = 10;

#[component]
fn BalanceFallback() -> impl IntoView {
    view! { <div class="w-1/4 rounded-full py-3 mt-1 bg-white/30 animate-pulse"></div> }
}

#[component]
pub fn Wallet() -> impl IntoView {
    let canisters = authenticated_canisters();
    let canisters_reader = move || MockPartialEq(canisters.get().and_then(|c| c.transpose()));
    let profile_details = create_resource(canisters_reader, move |canisters| async move {
        let canisters = try_or_redirect_opt!(canisters.0?);
        let user = canisters.authenticated_user();
        let user_details = user.get_profile_details().await.ok()?;
        Some(ProfileDetails::from(user_details))
    });
    let balance_resource = create_resource(canisters_reader, move |canisters| async move {
        let canisters = try_or_redirect_opt!(canisters.0?);
        let user = canisters.authenticated_user();
        let balance = user
            .get_utility_token_balance()
            .await
            .map(|b| b.to_string())
            .unwrap_or("Error".to_string());
        Some(balance)
    });
    let history_resource = create_resource(canisters_reader, move |canisters| async move {
        let canisters = try_or_redirect_opt!(canisters.0?);
        let history_prov = get_history_provider(canisters);
        let history = history_prov.get_history(0, RECENT_TXN_CNT).await.ok()?;

        Some(history)
    });

    view! {
        <div class="flex flex-col w-dvw min-h-dvh bg-black gap-12 px-4 py-4">
            <div class="grid grid-cols-2 grid-rows-1 items-center w-full">
                <Suspense fallback=FallbackGreeter>
                    {move || {
                        profile_details
                            .get()
                            .flatten()
                            .map(|details| view! { <ProfileGreeter details/> })
                            .unwrap_or_else(|| view! { <FallbackGreeter/> })
                    }}

                </Suspense>
            </div>
            <div class="flex flex-col w-full items-center mt-12 text-white">
                <span class="text-md lg:text-lg uppercase">Your Coin Balance</span>
                <Suspense fallback=BalanceFallback>
                    {move || {
                        balance_resource
                            .get()
                            .flatten()
                            .map(|bal| view! { <span class="text-xl lg:text-2xl">{bal}</span> })
                            .unwrap_or_else(|| {
                                view! {
                                    <span class="flex justify-center w-full">
                                        <BalanceFallback/>
                                    </span>
                                }
                            })
                    }}
                </Suspense>
            </div>
            <div class="flex flex-col w-full gap-2">
                <div class="flex flex-row w-full items-end justify-between">
                    <span class="text-white text-sm md:text-md">Recent Transactions</span>
                    // TODO: href
                    <a href="#" class="text-white/50 text-md md:text-lg">
                        See All
                    </a>
                </div>
                <div class="flex flex-col divide-y divide-white/10">
                    <Suspense fallback=BulletLoader>
                        {move || {
                            history_resource
                                .get()
                                .flatten()
                                .map(|history| {
                                    history
                                        .into_iter()
                                        .map(|info| view! { <TxnView info/> })
                                        .collect::<Vec<_>>()
                                })
                                .unwrap_or_else(|| vec![view! { <BulletLoader/> }])
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}

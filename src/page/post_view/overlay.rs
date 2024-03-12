use crate::{
    component::{modal::Modal, nav_icons::HomeSymbolFilled},
    state::canisters::{authenticated_canisters, Canisters},
    try_or_redirect_opt,
    utils::{
        web::{copy_to_clipboard, share_url},
        MockPartialEq,
    },
};
use leptos::*;
use leptos_icons::*;
use leptos_use::use_window;

use super::video_iter::{post_liked_by_me, PostDetails};
use candid::Principal;

#[component]
fn DisabledLikeButton() -> impl IntoView {
    view! {
        <button disabled>
            <Icon
                class="drop-shadow-lg text-neutral-400 animate-pulse"
                icon=icondata::AiHeartFilled
            />
        </button>
    }
}

#[component]
fn LikeButton(
    canisters: Canisters<true>,
    post_canister: Principal,
    post_id: u64,
    initial_liked: bool,
    likes: RwSignal<u64>,
) -> impl IntoView {
    let liked = create_rw_signal(initial_liked);
    let icon_class = Signal::derive(move || {
        if liked() {
            TextProp::from("fill-[url(#like-gradient)]")
        } else {
            TextProp::from("text-white")
        }
    });
    let icon_style = Signal::derive(move || {
        if liked() {
            Some(TextProp::from("filter: drop-shadow(2px 0 0 white) drop-shadow(-2px 0 0 white) drop-shadow(0 2px 0 white) drop-shadow(0 -2px 0 white);"))
        } else {
            None
        }
    });
    let like_toggle = create_action(move |&()| {
        let canisters = canisters.clone();
        batch(move || {
            if liked() {
                likes.update(|l| *l -= 1);
                liked.set(false)
            } else {
                likes.update(|l| *l += 1);
                liked.set(true);
            }
        });

        async move {
            let individual = canisters.individual_user(post_canister);
            match individual
                .update_post_toggle_like_status_by_caller(post_id)
                .await
            {
                Ok(_) => (),
                Err(e) => {
                    log::warn!("Error toggling like status: {:?}", e);
                    liked.update(|l| *l = !*l);
                }
            }
        }
    });

    view! {
        <svg style="width:0;height:0;position:absolute;" aria-hidden="true" focusable="false">
            <linearGradient id="like-gradient" x2="0" y2="1">
                <stop offset="0%" stop-color="#F74909"></stop>
                <stop offset="100%" stop-color="#FEBE48"></stop>
            </linearGradient>
        </svg>
        <button
            on:click=move |_| like_toggle.dispatch(())
            class="drop-shadow-lg"
            disabled=like_toggle.pending()
        >
            <Icon class=icon_class style=icon_style icon=icondata::AiHeartFilled/>
        </button>
    }
}

#[component]
pub fn VideoDetailsOverlay(post: PostDetails) -> impl IntoView {
    let show_share = create_rw_signal(false);
    let base_url = || {
        use_window()
            .as_ref()
            .and_then(|w| w.location().origin().ok())
    };
    let video_url = move || {
        base_url()
            .map(|b| format!("{b}/hot-or-not/{}/{}", post.canister_id, post.post_id))
            .unwrap_or_default()
    };

    let share = move || {
        let url = video_url();
        if share_url(&url).is_some() {
            return Some(());
        }
        show_share.set(true);
        Some(())
    };

    let auth_cans = authenticated_canisters();
    let auth_cans_reader = move || MockPartialEq(auth_cans.get().transpose());
    let liked_fetch = create_local_resource(auth_cans_reader, move |cans| async move {
        let canisters = try_or_redirect_opt!(cans.0)??;
        if let Some(liked) = post.liked_by_user {
            return Some((liked, canisters));
        }
        let liked = post_liked_by_me(&canisters, post.canister_id, post.post_id)
            .await
            .ok()?;
        Some((liked, canisters))
    });
    let profile_url = format!("/profile/{}", post.poster_principal.to_text());
    let likes = create_rw_signal(post.likes);

    view! {
        <div class="flex flex-row flex-nowrap justify-between items-end pb-16 px-2 md:px-6 w-full text-white absolute bottom-0 left-0 bg-transparent z-[4]">
            <div class="flex flex-col gap-2 w-9/12">
                <div class="flex flex-row items-center gap-2 min-w-0">
                    <a
                        href=profile_url
                        class="w-10 md:w-12 h-10 md:h-12 overflow-clip rounded-full border-white border-2"
                    >
                        <img class="h-full w-full object-cover" src=post.propic_url/>
                    </a>
                    <div class="flex flex-col w-7/12">
                        <span class="text-md md:text-lg font-bold truncate">
                            {post.display_name}
                        </span>
                        <span class="flex flex-row gap-1 items-center text-sm md:text-md">
                            <Icon icon=icondata::AiEyeOutlined/>
                            {post.views}
                        </span>
                    </div>
                </div>
                <span class="text-sm md:text-md ms-2 md:ms-4 w-full truncate">
                    {post.description}
                </span>
            </div>
            <div class="flex flex-col gap-6 items-end w-3/12 text-4xl">
                <a href="/refer-earn">
                    <Icon class="drop-shadow-lg" icon=icondata::AiGiftFilled/>
                </a>
                <div class="flex flex-col gap-1 items-center">
                    <Suspense fallback=DisabledLikeButton>
                        {move || {
                            liked_fetch()
                                .flatten()
                                .map(|(liked, canisters)| {
                                    view! {
                                        <LikeButton
                                            likes
                                            canisters
                                            post_canister=post.canister_id
                                            post_id=post.post_id
                                            initial_liked=liked
                                        />
                                    }
                                })
                        }}

                    </Suspense>
                    <span class="text-sm md:text-md">{likes}</span>
                </div>
                <button on:click=move |_| _ = share()>
                    <Icon class="drop-shadow-lg" icon=icondata::BsSendFill/>
                </button>
            </div>
        </div>
        <Modal show=show_share>
            <div class="flex flex-col justify-center items-center gap-4 text-white">
                <span class="text-lg">Share</span>
                <div class="flex flex-row w-full gap-2">
                    <p class="text-md max-w-full bg-white/10 rounded-full p-2 overflow-x-scroll whitespace-nowrap">
                        {video_url}
                    </p>
                    <button on:click=move |_| _ = copy_to_clipboard(&video_url())>
                        <Icon class="text-xl" icon=icondata::FaCopyRegular/>
                    </button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
pub fn HomeButtonOverlay() -> impl IntoView {
    view! {
        <div class="flex w-full items-center justify-center pt-4 absolute top-0 left-0 bg-transparent z-[4]">
            <div class="rounded-full p-2 text-white bg-black/20">
                <div class="flex flex-row items-center gap-1 py-2 px-6 rounded-full bg-orange-500">
                    <Icon class="w-3 h-3" icon=HomeSymbolFilled/>
                    <span>Home</span>
                </div>
            </div>
        </div>
    }
}

use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Health
        crate::handlers::health::health_live,
        crate::handlers::health::health_check,
        // Auth
        crate::handlers::auth::register,
        crate::handlers::auth::login,
        crate::handlers::auth::refresh,
        crate::handlers::auth::logout,
        crate::handlers::auth::change_password,
        crate::handlers::auth::forgot_password,
        crate::handlers::auth::verify_reset_code,
        crate::handlers::auth::reset_password,
        crate::handlers::auth::verify_email,
        crate::handlers::auth::resend_verification,
        crate::handlers::auth::google_auth,
        crate::handlers::auth::apple_auth,
        // Users
        crate::handlers::users::get_profile,
        crate::handlers::users::update_profile,
        crate::handlers::users::export_data,
        crate::handlers::users::delete_account,
        crate::handlers::users::request_email_change,
        // Items
        crate::handlers::items::create_item,
        crate::handlers::items::list_items,
        crate::handlers::items::get_item,
        crate::handlers::items::update_item,
        crate::handlers::items::delete_item,
        crate::handlers::items::claim_item,
        crate::handlers::items::unclaim_item,
        crate::handlers::items::owner_unclaim_web,
        crate::handlers::items::batch_delete,
        // Categories
        crate::handlers::categories::list_categories,
        // Friends
        crate::handlers::friends::search_users,
        crate::handlers::friends::send_request,
        crate::handlers::friends::list_pending,
        crate::handlers::friends::list_sent,
        crate::handlers::friends::cancel_request,
        crate::handlers::friends::accept_request,
        crate::handlers::friends::decline_request,
        crate::handlers::friends::list_friends,
        crate::handlers::friends::remove_friend,
        // Circles
        crate::handlers::circles::create_circle,
        crate::handlers::circles::list_circles,
        crate::handlers::circles::get_circle,
        crate::handlers::circles::update_circle,
        crate::handlers::circles::delete_circle,
        crate::handlers::circles::create_direct_circle,
        crate::handlers::circles::add_member,
        crate::handlers::circles::create_invite,
        crate::handlers::circles::join_via_invite,
        crate::handlers::circles::remove_member,
        crate::handlers::circles::list_invites,
        crate::handlers::circles::revoke_invite,
        crate::handlers::circles::share_item,
        crate::handlers::circles::batch_share_items,
        crate::handlers::circles::get_share_rule,
        crate::handlers::circles::set_share_rule,
        crate::handlers::circles::list_circle_items,
        crate::handlers::circles::get_circle_item,
        crate::handlers::circles::unshare_item,
        crate::handlers::circles::get_feed,
        crate::handlers::circles::transfer_ownership,
        crate::handlers::circles::list_reservations,
        crate::handlers::circles::list_my_share_rules,
        // Community wishes (Entraide)
        crate::handlers::community_wishes::create_wish,
        crate::handlers::community_wishes::list_wishes,
        crate::handlers::community_wishes::get_wish,
        crate::handlers::community_wishes::list_my_wishes,
        crate::handlers::community_wishes::list_my_offers,
        crate::handlers::community_wishes::list_recent_fulfilled,
        crate::handlers::community_wishes::update_wish,
        crate::handlers::community_wishes::close_wish,
        crate::handlers::community_wishes::delete_wish,
        crate::handlers::community_wishes::reopen_wish,
        crate::handlers::community_wishes::offer_wish,
        crate::handlers::community_wishes::withdraw_offer,
        crate::handlers::community_wishes::reject_offer,
        crate::handlers::community_wishes::confirm_wish,
        crate::handlers::community_wishes::report_wish,
        crate::handlers::community_wishes::block_wish,
        crate::handlers::community_wishes::unblock_wish,
        // Wish messages
        crate::handlers::wish_messages::send_message,
        crate::handlers::wish_messages::list_messages,
        // Notifications
        crate::handlers::notifications::list_notifications,
        crate::handlers::notifications::mark_all_read,
        crate::handlers::notifications::mark_read,
        crate::handlers::notifications::delete_notification,
        crate::handlers::notifications::unread_count,
        // Push tokens
        crate::handlers::push_tokens::register_token,
        crate::handlers::push_tokens::unregister_token,
        // Share links
        crate::handlers::share_links::create_share_link,
        crate::handlers::share_links::list_share_links,
        crate::handlers::share_links::delete_share_link,
        crate::handlers::share_links::update_share_link,
        // Upload
        crate::handlers::upload::upload_image,
        // Shared
        crate::handlers::shared::get_shared_view,
        crate::handlers::shared::claim_via_share,
        crate::handlers::shared::unclaim_via_share,
        crate::handlers::shared::web_claim_via_share,
        crate::handlers::shared::web_unclaim_via_share,
        // Admin
        crate::handlers::admin::list_pending,
        crate::handlers::admin::approve_wish,
        crate::handlers::admin::reject_wish,
    ),
    info(
        title = "Offrii API",
        version = "1.0.0",
        description = "Offrii — Offre, partage, fais plaisir.\n\nAPI REST pour l'application Offrii.",
        contact(name = "Offrii", url = "https://offrii.com")
    ),
    tags(
        (name = "Health", description = "Health checks"),
        (name = "Auth", description = "Authentication & account management"),
        (name = "Items", description = "Wishlist items (envies)"),
        (name = "Categories", description = "Item categories"),
        (name = "Users", description = "User profile management"),
        (name = "Circles", description = "Circles (proches) — groups for sharing"),
        (name = "Friends", description = "Friend requests & friendships"),
        (name = "Entraide", description = "Community mutual aid (besoins)"),
        (name = "Messages", description = "Entraide wish messages"),
        (name = "Notifications", description = "In-app notification center"),
        (name = "PushTokens", description = "APNs push token management"),
        (name = "ShareLinks", description = "Share link management"),
        (name = "Shared", description = "Public shared views"),
        (name = "Upload", description = "Image upload"),
        (name = "Admin", description = "Admin moderation panel")
    ),
    modifiers(&SecurityAddon),
)]
pub struct ApiDoc;

struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearer_auth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}

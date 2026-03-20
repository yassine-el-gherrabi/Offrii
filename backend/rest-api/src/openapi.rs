use utoipa::OpenApi;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};

#[derive(OpenApi)]
#[openapi(
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

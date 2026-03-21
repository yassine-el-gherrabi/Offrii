use std::time::Duration;

use async_trait::async_trait;
use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;

use crate::errors::AppError;
use crate::traits;

/// Backoff delays between retry attempts (1 s, then 2 s).
const RETRY_DELAYS: [Duration; 2] = [Duration::from_secs(1), Duration::from_secs(2)];

/// Maximum number of send attempts (initial + retries).
const MAX_ATTEMPTS: usize = 3;

// ── Brand constants ─────────────────────────────────────────────────

const LOGO_URL: &str = "https://cdn.offrii.com/branding/logo-1024.png";

/// Wraps email body content in the Offrii branded template.
/// Table-based layout for maximum email client compatibility.
/// All CSS is inline (Gmail strips <style> blocks).
fn email_template(body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="color-scheme" content="light">
<meta name="supported-color-schemes" content="light">
<title>Offrii</title>
</head>
<body style="margin:0;padding:0;background-color:#FFFAF9;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;-webkit-font-smoothing:antialiased;">
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="background-color:#FFFAF9;">
<tr><td align="center" style="padding:32px 16px 0;">
<!-- Logo -->
<table role="presentation" cellpadding="0" cellspacing="0">
<tr><td align="center" style="padding-bottom:8px;">
<img src="{LOGO_URL}" alt="Offrii" width="56" height="56" style="display:block;border-radius:14px;border:0;" />
</td></tr>
<tr><td align="center" style="padding-bottom:24px;">
<span style="font-size:20px;font-weight:700;color:#FF6B6B;letter-spacing:-0.02em;">Offrii</span>
</td></tr>
</table>
</td></tr>
<tr><td align="center" style="padding:0 16px;">
<!-- Card -->
<table role="presentation" width="100%" cellpadding="0" cellspacing="0" style="max-width:520px;background-color:#ffffff;border-radius:16px;overflow:hidden;">
<tr><td style="padding:32px 32px 24px;">
{body}
</td></tr>
</table>
</td></tr>
<tr><td align="center" style="padding:24px 16px 32px;">
<!-- Footer -->
<table role="presentation" cellpadding="0" cellspacing="0" style="max-width:520px;">
<tr><td align="center" style="font-size:12px;color:#9ca3af;line-height:1.6;">
<span style="color:#FF6B6B;font-weight:600;">Offrii</span> — Offre, partage, fais plaisir.<br>
Vous recevez cet email car vous avez un compte Offrii.<br>
<a href="https://offrii.com" style="color:#9ca3af;text-decoration:underline;">offrii.com</a>
</td></tr>
</table>
</td></tr>
</table>
</body>
</html>"#
    )
}

/// Primary CTA button (corail).
fn cta_button(label: &str, url: &str) -> String {
    format!(
        r#"<table role="presentation" cellpadding="0" cellspacing="0" width="100%" style="padding-top:24px;">
<tr><td align="center">
<a href="{url}" style="display:inline-block;background-color:#FF6B6B;color:#ffffff;font-size:16px;font-weight:600;text-decoration:none;padding:14px 32px;border-radius:12px;mso-padding-alt:0;text-align:center;">
<!--[if mso]><i style="mso-font-width:150%;mso-text-raise:27pt">&nbsp;</i><![endif]-->
<span style="mso-text-raise:13pt;">{label}</span>
<!--[if mso]><i style="mso-font-width:150%;">&nbsp;</i><![endif]-->
</a>
</td></tr>
</table>"#
    )
}

// ── Service ─────────────────────────────────────────────────────────

pub struct ResendEmailService {
    client: Resend,
    from: String,
    base_url: String,
}

impl ResendEmailService {
    pub fn new(api_key: &str, from: String, base_url: String) -> Self {
        Self {
            client: Resend::new(api_key),
            from,
            base_url,
        }
    }

    /// Send an email with retry + exponential backoff.
    /// Tries up to [`MAX_ATTEMPTS`] times with delays defined in [`RETRY_DELAYS`].
    async fn send_with_retry(&self, email: CreateEmailBaseOptions) -> Result<(), AppError> {
        let mut last_err = None;
        for attempt in 0..MAX_ATTEMPTS {
            match self.client.emails.send(email.clone()).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    tracing::warn!(
                        attempt = attempt + 1,
                        max = MAX_ATTEMPTS,
                        error = %e,
                        "email send failed, will retry"
                    );
                    last_err = Some(e);
                    if let Some(delay) = RETRY_DELAYS.get(attempt) {
                        tokio::time::sleep(*delay).await;
                    }
                }
            }
        }
        Err(AppError::Internal(anyhow::anyhow!(
            "email send failed after {MAX_ATTEMPTS} attempts: {}",
            last_err.map(|e| e.to_string()).unwrap_or_default()
        )))
    }
}

#[async_trait]
impl traits::EmailService for ResendEmailService {
    async fn send_password_reset_code(&self, to: &str, code: &str) -> Result<(), AppError> {
        // Space out each character for OTP-style display
        let spaced: String = code
            .chars()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" ");

        let body = format!(
            r#"<h1 style="margin:0 0 8px;font-size:22px;font-weight:700;color:#1a1a2e;">
Votre code de réinitialisation
</h1>
<p style="margin:0 0 24px;font-size:15px;color:#6b7280;line-height:1.6;">
Utilisez le code ci-dessous pour réinitialiser votre mot de passe.
</p>
<table role="presentation" cellpadding="0" cellspacing="0" width="100%">
<tr><td align="center" style="padding:20px 0;">
<div style="display:inline-block;background-color:#f3f4f6;border-radius:12px;padding:16px 32px;font-family:'SF Mono',SFMono-Regular,Consolas,'Liberation Mono',Menlo,monospace;font-size:32px;font-weight:700;letter-spacing:8px;color:#1a1a2e;">
{spaced}
</div>
</td></tr>
</table>
<p style="margin:16px 0 0;font-size:13px;color:#9ca3af;line-height:1.5;">
Ce code expire dans <strong>30 minutes</strong>.<br>
Si vous n'avez pas demandé de réinitialisation, ignorez cet email.
</p>"#
        );

        let html = email_template(&body);
        let email =
            CreateEmailBaseOptions::new(&self.from, [to], "Votre code Offrii").with_html(&html);

        self.send_with_retry(email).await
    }

    async fn send_welcome_email(
        &self,
        to: &str,
        display_name: Option<&str>,
    ) -> Result<(), AppError> {
        let name = display_name.unwrap_or("là");

        let body = format!(
            r#"<h1 style="margin:0 0 8px;font-size:22px;font-weight:700;color:#1a1a2e;">
Bienvenue {name} !
</h1>
<p style="margin:0 0 24px;font-size:15px;color:#6b7280;line-height:1.6;">
Votre compte est prêt. Voici ce que vous pouvez faire sur Offrii :
</p>
<table role="presentation" cellpadding="0" cellspacing="0" width="100%">
<tr><td style="padding:8px 0;font-size:15px;color:#1a1a2e;">
<strong>Envies</strong> — Créez votre liste et gardez une trace de tout ce qui vous fait envie
</td></tr>
<tr><td style="padding:8px 0;font-size:15px;color:#1a1a2e;">
<strong>Proches</strong> — Partagez vos envies avec vos amis et votre famille
</td></tr>
<tr><td style="padding:8px 0;font-size:15px;color:#1a1a2e;">
<strong>Entraide</strong> — Publiez un besoin ou proposez votre aide à la communauté
</td></tr>
</table>
<p style="margin:20px 0 0;font-size:13px;color:#9ca3af;text-align:center;">
A très vite sur Offrii !
</p>"#,
        );

        let html = email_template(&body);
        let subject = if name == "là" {
            "Bienvenue sur Offrii".to_string()
        } else {
            format!("Bienvenue sur Offrii, {name}")
        };
        let email = CreateEmailBaseOptions::new(&self.from, [to], &subject).with_html(&html);

        self.send_with_retry(email).await
    }

    async fn send_welcome_and_verify_email(
        &self,
        to: &str,
        display_name: Option<&str>,
        token: &str,
    ) -> Result<(), AppError> {
        let name = display_name.unwrap_or("là");
        let verification_url = format!("{}/v1/auth/verify-email?token={token}", self.base_url);

        let body = format!(
            r#"<h1 style="margin:0 0 8px;font-size:22px;font-weight:700;color:#1a1a2e;">
Bienvenue {name} !
</h1>
<p style="margin:0 0 24px;font-size:15px;color:#6b7280;line-height:1.6;">
Votre compte est prêt. Confirmez votre adresse email pour accéder à toutes les fonctionnalités.
</p>
{cta}
<p style="margin:24px 0 0;font-size:15px;color:#6b7280;line-height:1.6;">
Voici ce que vous pouvez faire sur Offrii :
</p>
<table role="presentation" cellpadding="0" cellspacing="0" width="100%">
<tr><td style="padding:8px 0;font-size:15px;color:#1a1a2e;">
<strong>Envies</strong> — Créez votre liste et gardez une trace de tout ce qui vous fait envie
</td></tr>
<tr><td style="padding:8px 0;font-size:15px;color:#1a1a2e;">
<strong>Proches</strong> — Partagez vos envies avec vos amis et votre famille
</td></tr>
<tr><td style="padding:8px 0;font-size:15px;color:#1a1a2e;">
<strong>Entraide</strong> — Publiez un besoin ou proposez votre aide à la communauté
</td></tr>
</table>
<p style="margin:20px 0 0;font-size:13px;color:#9ca3af;line-height:1.5;text-align:center;">
Ou copiez ce lien dans votre navigateur :<br>
<a href="{verification_url}" style="color:#FF6B6B;text-decoration:underline;word-break:break-all;font-size:12px;">{verification_url}</a>
</p>
<p style="margin:8px 0 0;font-size:13px;color:#9ca3af;line-height:1.5;">
Ce lien expire dans <strong>24 heures</strong>.
</p>"#,
            cta = cta_button("Vérifier mon email", &verification_url)
        );

        let html = email_template(&body);
        let subject = if name == "là" {
            "Bienvenue sur Offrii — Vérifiez votre email".to_string()
        } else {
            format!("Bienvenue {name} — Vérifiez votre email")
        };
        let email = CreateEmailBaseOptions::new(&self.from, [to], &subject).with_html(&html);

        self.send_with_retry(email).await
    }

    async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), AppError> {
        let verification_url = format!("{}/v1/auth/verify-email?token={token}", self.base_url);

        let body = format!(
            r#"<h1 style="margin:0 0 8px;font-size:22px;font-weight:700;color:#1a1a2e;">
Vérifiez votre email
</h1>
<p style="margin:0 0 4px;font-size:15px;color:#6b7280;line-height:1.6;">
Cliquez sur le bouton ci-dessous pour confirmer votre adresse email et accéder à toutes les fonctionnalités d'Offrii.
</p>
{cta}
<p style="margin:20px 0 0;font-size:13px;color:#9ca3af;line-height:1.5;text-align:center;">
Ou copiez ce lien dans votre navigateur :<br>
<a href="{verification_url}" style="color:#FF6B6B;text-decoration:underline;word-break:break-all;font-size:12px;">{verification_url}</a>
</p>
<p style="margin:16px 0 0;font-size:13px;color:#9ca3af;line-height:1.5;">
Ce lien expire dans <strong>24 heures</strong>.<br>
Si vous n'avez pas créé de compte Offrii, ignorez cet email.
</p>"#,
            cta = cta_button("Vérifier mon email", &verification_url)
        );

        let html = email_template(&body);
        let email = CreateEmailBaseOptions::new(&self.from, [to], "Vérifiez votre email — Offrii")
            .with_html(&html);

        self.send_with_retry(email).await
    }

    async fn send_password_changed_email(&self, to: &str) -> Result<(), AppError> {
        let body = r#"<h1 style="margin:0 0 8px;font-size:22px;font-weight:700;color:#1a1a2e;">
Mot de passe modifié
</h1>
<p style="margin:0 0 16px;font-size:15px;color:#6b7280;line-height:1.6;">
Votre mot de passe Offrii a été modifié avec succès.
</p>
<p style="margin:0 0 0;font-size:13px;color:#9ca3af;line-height:1.5;">
Si vous n'êtes pas à l'origine de ce changement, contactez-nous immédiatement à
<a href="mailto:contact@offrii.com" style="color:#FF6B6B;text-decoration:underline;">contact@offrii.com</a>
</p>"#;

        let html = email_template(body);
        let email = CreateEmailBaseOptions::new(&self.from, [to], "Mot de passe modifié — Offrii")
            .with_html(&html);

        self.send_with_retry(email).await
    }

    async fn send_email_change_verification(&self, to: &str, token: &str) -> Result<(), AppError> {
        let verify_url = format!(
            "{}/v1/auth/verify-email-change?token={token}",
            self.base_url
        );

        let body = format!(
            r#"<h1 style="margin:0 0 8px;font-size:22px;font-weight:700;color:#1a1a2e;">
Confirmez votre nouvel email
</h1>
<p style="margin:0 0 4px;font-size:15px;color:#6b7280;line-height:1.6;">
Vous avez demandé à changer votre adresse email Offrii. Cliquez sur le bouton ci-dessous pour confirmer cette nouvelle adresse.
</p>
{cta}
<p style="margin:20px 0 0;font-size:13px;color:#9ca3af;line-height:1.5;text-align:center;">
Ou copiez ce lien dans votre navigateur :<br>
<a href="{verify_url}" style="color:#FF6B6B;text-decoration:underline;word-break:break-all;font-size:12px;">{verify_url}</a>
</p>
<p style="margin:16px 0 0;font-size:13px;color:#9ca3af;line-height:1.5;">
Ce lien expire dans <strong>1 heure</strong>.<br>
Si vous n'avez pas demandé ce changement, ignorez cet email.
</p>"#,
            cta = cta_button("Confirmer mon nouvel email", &verify_url)
        );

        let html = email_template(&body);
        let email =
            CreateEmailBaseOptions::new(&self.from, [to], "Confirmez votre nouvel email — Offrii")
                .with_html(&html);

        self.send_with_retry(email).await
    }

    async fn send_email_changed_notification(
        &self,
        to: &str,
        new_email: &str,
    ) -> Result<(), AppError> {
        let body = format!(
            r#"<h1 style="margin:0 0 8px;font-size:22px;font-weight:700;color:#1a1a2e;">
Adresse email modifiée
</h1>
<p style="margin:0 0 16px;font-size:15px;color:#6b7280;line-height:1.6;">
Votre adresse email Offrii a été modifiée vers <strong>{new_email}</strong>.
</p>
<p style="margin:0 0 0;font-size:13px;color:#9ca3af;line-height:1.5;">
Si vous n'êtes pas à l'origine de ce changement, contactez-nous immédiatement à
<a href="mailto:contact@offrii.com" style="color:#FF6B6B;text-decoration:underline;">contact@offrii.com</a>
</p>"#
        );

        let html = email_template(&body);
        let email =
            CreateEmailBaseOptions::new(&self.from, [to], "Adresse email modifiée — Offrii")
                .with_html(&html);

        self.send_with_retry(email).await
    }

    async fn send_inactivity_warning(&self, to: &str) -> Result<(), AppError> {
        let body = r#"<h1 style="margin:0 0 8px;font-size:22px;font-weight:700;color:#1a1a2e;">
Votre compte Offrii sera bientôt supprimé
</h1>
<p style="margin:0 0 16px;font-size:15px;color:#6b7280;line-height:1.6;">
Votre compte Offrii est inactif depuis plus de 23 mois. Sans connexion dans les 30 prochains jours, votre compte et toutes vos données seront définitivement supprimés.
</p>
<p style="margin:0 0 0;font-size:15px;color:#6b7280;line-height:1.6;">
Pour conserver votre compte, il vous suffit de vous connecter à l'application.
</p>"#;

        let html = email_template(body);
        let email = CreateEmailBaseOptions::new(
            &self.from,
            [to],
            "Votre compte Offrii sera bientôt supprimé",
        )
        .with_html(&html);

        self.send_with_retry(email).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_delays_match_max_attempts() {
        // We need exactly MAX_ATTEMPTS - 1 delays (no delay after the last attempt).
        assert_eq!(
            RETRY_DELAYS.len(),
            MAX_ATTEMPTS - 1,
            "should have one fewer delay than attempts"
        );
    }

    #[test]
    fn retry_delays_are_increasing() {
        for window in RETRY_DELAYS.windows(2) {
            assert!(
                window[1] > window[0],
                "delays must increase: {:?} should be > {:?}",
                window[1],
                window[0]
            );
        }
    }

    #[test]
    fn email_template_contains_logo() {
        let html = email_template("test body");
        assert!(
            html.contains(LOGO_URL),
            "template must reference the brand logo"
        );
    }

    #[test]
    fn email_template_wraps_body_content() {
        let html = email_template("<p>Hello</p>");
        assert!(html.contains("<p>Hello</p>"), "body must be embedded");
        assert!(html.contains("<!DOCTYPE html>"), "must be full HTML doc");
    }

    #[test]
    fn cta_button_contains_link_and_label() {
        let btn = cta_button("Click me", "https://example.com");
        assert!(btn.contains("https://example.com"));
        assert!(btn.contains("Click me"));
    }

    #[test]
    fn max_attempts_is_at_least_two() {
        const {
            assert!(
                MAX_ATTEMPTS >= 2,
                "retry logic requires at least 2 attempts to be useful"
            )
        };
    }
}

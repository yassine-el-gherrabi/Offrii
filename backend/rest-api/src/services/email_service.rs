use async_trait::async_trait;
use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;

use crate::errors::AppError;
use crate::traits;

// ── Brand constants ─────────────────────────────────────────────────

const LOGO_URL: &str = "https://pub-83ca22acc7354445815c6b4e152ba243.r2.dev/branding/logo-1024.png";

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
}

impl ResendEmailService {
    pub fn new(api_key: &str, from: String) -> Self {
        Self {
            client: Resend::new(api_key),
            from,
        }
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

        self.client
            .emails
            .send(email)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("email send failed: {e}")))?;

        Ok(())
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
<span style="font-size:20px;">🎁</span>&nbsp;&nbsp;<strong>Envies</strong> — Créez votre liste et gardez une trace de tout ce qui vous fait envie
</td></tr>
<tr><td style="padding:8px 0;font-size:15px;color:#1a1a2e;">
<span style="font-size:20px;">👥</span>&nbsp;&nbsp;<strong>Proches</strong> — Partagez vos envies avec vos amis et votre famille
</td></tr>
<tr><td style="padding:8px 0;font-size:15px;color:#1a1a2e;">
<span style="font-size:20px;">🤝</span>&nbsp;&nbsp;<strong>Entraide</strong> — Publiez un besoin ou proposez votre aide à la communauté
</td></tr>
</table>
{cta}
<p style="margin:20px 0 0;font-size:13px;color:#9ca3af;text-align:center;">
À très vite sur Offrii !
</p>"#,
            cta = cta_button(
                "Ouvrir Offrii",
                "https://apps.apple.com/app/offrii/id0000000000"
            )
        );

        let html = email_template(&body);
        let subject = if name == "là" {
            "Bienvenue sur Offrii !".to_string()
        } else {
            format!("Bienvenue sur Offrii, {name} !")
        };
        let email = CreateEmailBaseOptions::new(&self.from, [to], &subject).with_html(&html);

        self.client
            .emails
            .send(email)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("email send failed: {e}")))?;

        Ok(())
    }

    async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), AppError> {
        let verification_url = format!("https://offrii.com/verify?token={token}");

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
        let email = CreateEmailBaseOptions::new(
            &self.from,
            [to],
            "\u{2709}\u{fe0f} Vérifiez votre email — Offrii",
        )
        .with_html(&html);

        self.client
            .emails
            .send(email)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("email send failed: {e}")))?;

        Ok(())
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
<a href="mailto:yassineelgherrabi@gmail.com" style="color:#FF6B6B;text-decoration:underline;">yassineelgherrabi@gmail.com</a>
</p>"#;

        let html = email_template(body);
        let email = CreateEmailBaseOptions::new(
            &self.from,
            [to],
            "\u{1f512} Mot de passe modifié — Offrii",
        )
        .with_html(&html);

        self.client
            .emails
            .send(email)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("email send failed: {e}")))?;

        Ok(())
    }
}

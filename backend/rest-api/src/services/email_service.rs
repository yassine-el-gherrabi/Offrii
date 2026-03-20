use async_trait::async_trait;
use resend_rs::Resend;
use resend_rs::types::CreateEmailBaseOptions;

use crate::errors::AppError;
use crate::traits;

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
        let email =
            CreateEmailBaseOptions::new(&self.from, [to], "Your Offrii password reset code")
                .with_html(&format!(
                    "<h2>Password Reset</h2>\
             <p>Your code: <strong>{code}</strong></p>\
             <p>Valid for 30 minutes.</p>"
                ));

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
        let name = display_name.unwrap_or("there");
        let email = CreateEmailBaseOptions::new(&self.from, [to], "Welcome to Offrii!")
            .with_html(&format!(
                "<h2>Welcome to Offrii, {name}!</h2>\
                 <p>Your account is ready. Start creating your wishlist and sharing it with your loved ones.</p>\
                 <p>See you soon on Offrii!</p>"
            ));

        self.client
            .emails
            .send(email)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("email send failed: {e}")))?;

        Ok(())
    }

    async fn send_verification_email(&self, to: &str, token: &str) -> Result<(), AppError> {
        let verification_url = format!("https://offrii.com/verify?token={token}");
        let email =
            CreateEmailBaseOptions::new(&self.from, [to], "Verify your Offrii email address")
                .with_html(&format!(
                    "<h2>Verify your email</h2>\
             <p>Click the link below to verify your email address:</p>\
             <p><a href=\"{verification_url}\">{verification_url}</a></p>\
             <p>This link expires in 24 hours.</p>"
                ));

        self.client
            .emails
            .send(email)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("email send failed: {e}")))?;

        Ok(())
    }
}

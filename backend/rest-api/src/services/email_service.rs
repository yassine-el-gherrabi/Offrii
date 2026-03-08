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
}

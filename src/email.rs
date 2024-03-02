use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use lettre::message::header::ContentType;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use serde::{Deserialize, Serialize};
use crate::repository::{Repository, RepositoryError, RepositoryItem, RepositoryResult};

#[derive(Clone)]
pub struct EmailRepository {
    smtp_username: String,
    smtp_token: String,
    smtp_domain: String,
}

impl EmailRepository {
    pub fn new(
        smtp_username: String,
        smtp_token: String,
        smtp_domain: String,
    ) -> Self {
        EmailRepository {
            smtp_username,
            smtp_token,
            smtp_domain
        }
    }

    pub async fn send_auth_token(&self, email: &str, token: &str) -> RepositoryResult<()> {
        // TODO: produce a link back to the app or send it to this method
        let message = format!("Your auth token is: {}", token);
        self.send_email("Fe-Vault Login Token", email, &message).await
    }

    pub async fn send_email(&self, subject: &str, email: &str, message: &str) -> RepositoryResult<()> {
        let creds = Credentials::new(self.smtp_username.clone(), self.smtp_token.clone());

        let mailer: AsyncSmtpTransport<Tokio1Executor> =
            AsyncSmtpTransport::<Tokio1Executor>::relay("smtp.mailgun.org")
                .unwrap()
                .credentials(creds)
                .build();

        let from: Mailbox = "Fe-Vault <noreply@fe-vault.thehe.art>".parse().unwrap();
        log::info!("sending from: {}", from.email);
        // TODO: handle unwrap
        let message = Message::builder()
            .from(from)
            .to(email.parse().unwrap())
            .subject(subject)
            .header(ContentType::TEXT_PLAIN)
            .body(message.to_string())
            .unwrap();

        log::info!("Sending email to {}.", email);
        if let Err(e) = mailer.send(message).await {
            log::error!("Failed to send email: {}", e);
            return Err(RepositoryError::Other);
        }

        Ok(())
    }
}

#[shuttle_runtime::async_trait]
impl Repository for EmailRepository {
    type Item = MailerStatus;
    type RowIdentifier = i64;

    // TODO: make default implementations for cases where the database is not used beyond updating
    // TODO: ... the user's token once default type implementations for traits is stable
    async fn save(&self, _: MailerStatus) -> RepositoryResult<i64> {
        Ok(0)
    }

    async fn get(&self, _: i64) -> RepositoryResult<MailerStatus> {
        Ok(MailerStatus::Idle)
    }

    async fn get_all(&self) -> RepositoryResult<Vec<MailerStatus>> {
        Ok(vec![MailerStatus::Idle])
    }

    async fn delete(&self, _: i64) -> RepositoryResult<()> {
        Ok(())
    }

    async fn create_table(&self) -> RepositoryResult<()> {
        Ok(())
    }

    async fn drop_table(&self) -> RepositoryResult<()> {
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum MailerStatus {
    Idle,
    Error,
}

impl RepositoryItem for MailerStatus {
    fn masked_columns(_: bool) -> Vec<String> {
        Self::all_columns()
    }

    fn saved_columns() -> Vec<String> {
        Self::all_columns()
    }

    fn all_columns() -> Vec<String> {
        vec![]
    }

    fn table_name() -> String where Self: Sized {
        "email_manager".to_string()
    }
}
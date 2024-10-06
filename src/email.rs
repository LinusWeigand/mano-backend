use lettre::transport::smtp::authentication::Credentials;
use lettre::{Address, Message, Transport};
use lettre::transport::smtp::{SmtpTransport, Error as SmtpError};
use thiserror::Error;
pub struct EmailManager {
    email: String,
    smtp_transport: SmtpTransport,
}

#[derive(Error,Debug)]
pub enum EmailManagerError {
    #[error("SMTP error: {0}")]
    Smtp(#[from] SmtpError),
    #[error("Invalid email address: {0}")]
    InvalidEmail(#[from] lettre::address::AddressError),
    #[error("Faild to build email address: {0}")]
    BuildMessage(#[from] lettre::error::Error),
}


impl EmailManager {
    pub fn new(smtp_email: &str, smtp_password: &str) -> Result<Self, SmtpError> {

        let creds = Credentials::new(smtp_email.to_string(), smtp_password.to_string());
        let mailer = SmtpTransport::relay("smtp.gmail.com")?
            .credentials(creds)
            .build();
        Ok(EmailManager {
            smtp_transport: mailer,
            email: smtp_email.to_string(),
        })
    }

    pub fn send_email(&self, email: &str, subject: &str, body: &str) -> Result<(), EmailManagerError> {
        let from_address: Address = self.email.parse()?;
        let to_address: Address  = email.parse()?;

        let email = Message::builder()
            .from(from_address.into())
            .to(to_address.into())
            .subject(subject)
            .body(body.to_string())?;

        self.smtp_transport.send(&email)?;
        Ok(())
    }
}
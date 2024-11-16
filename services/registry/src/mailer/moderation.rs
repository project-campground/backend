use anyhow::Result;
use mailgun_rs::{EmailAddress, Mailgun, MailgunRegion, Message as MailgunMessage};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message as LettreMessage, SmtpTransport, Transport};
use crate::config::{MODERATION_EMAIL_CONFIG, MailConfig};

pub struct HtmlMailOpts {
    pub to: String,
    pub subject: String,
}

pub struct ModerationMailer {}

impl ModerationMailer {
    pub async fn send_template<T: askama::Template>(opts: HtmlMailOpts, template: T) -> Result<()> {
        let HtmlMailOpts { to, subject } = opts;

        match &*MODERATION_EMAIL_CONFIG {
            MailConfig::Mailgun {
                api_key,
                domain,
                from_name,
                from_address,
            } => {
                let recipient = EmailAddress::address(&to);
                let message = MailgunMessage {
                    to: vec![recipient],
                    subject,
                    html: template.render().unwrap(),
                    ..Default::default()
                };
    
                let client = Mailgun {
                    api_key: api_key.clone(),
                    domain: domain.clone(),
                    message,
                };
                let sender = EmailAddress::name_address(&from_name, &from_address);
    
                client.async_send(MailgunRegion::US, &sender).await?;
                return Ok(())
            },
            MailConfig::SMTP {
                host,
                username,
                password,
                from_address,
            } => {
                let recipient = LettreMessage::builder()
                    .from(from_address.parse::<lettre::message::Mailbox>().unwrap())
                    .to(to.parse::<lettre::message::Mailbox>().unwrap())
                    .subject(subject)
                    .body(template.render().unwrap())
                    .unwrap();
    
                let creds = Credentials::new(username.to_owned(), password.to_owned());
    
                let mailer = SmtpTransport::relay(&host)
                    .unwrap()
                    .credentials(creds)
                    .build();
    
                mailer.send(&recipient).unwrap();
            }
        }
        Ok(())
    }
}
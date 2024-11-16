pub mod moderation;

use anyhow::Result;
use mailgun_rs::{EmailAddress, Mailgun, MailgunRegion, Message as MailgunMessage};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message as LettreMessage, SmtpTransport, Transport};
use crate::config::{EMAIL_CONFIG, MailConfig};
use serde::{Deserialize, Serialize};
use askama::Template;

#[derive(Template)]
#[template(path = "confirm_email.html")]
struct ConfirmEmailTemplate<'a> {
    identifier: &'a str,
    token: &'a str,
}

#[derive(Template)]
#[template(path = "password_reset.html")]
struct PasswordResetTemplate<'a> {
    identifier: &'a str,
    token: &'a str,
}

#[derive(Template)]
#[template(path = "delete_account.html")]
struct DeleteAccountTemplate<'a> {
    identifier: &'a str,
    token: &'a str,
}

#[derive(Template)]
#[template(path = "update_email.html")]
struct UpdateEmailTemplate<'a> {
    identifier: &'a str,
    token: &'a str,
}

pub struct MailOpts {
    pub to: String,
    pub subject: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IdentifierAndTokenParams {
    pub identifier: String,
    pub token: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TokenParam {
    pub token: String,
}

pub async fn send_template<T: Template>(opts: MailOpts, template: &T) -> Result<()> {
    let MailOpts {
        to,
        subject,
    } = opts;

    match &*EMAIL_CONFIG {
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

pub async fn send_reset_password(to: String, params: IdentifierAndTokenParams) -> Result<()> {
    let template = PasswordResetTemplate {
        identifier: &params.identifier,
        token: &params.token,
    };
    send_template(MailOpts {
        to,
        subject: "Password Reset Requested".to_string(),
    }, &template)
    .await
}

pub async fn send_account_delete(to: String, params: IdentifierAndTokenParams) -> Result<()> {
    let template = DeleteAccountTemplate {
        identifier: &params.identifier,
        token: &params.token,
    };
    send_template(MailOpts {
        to,
        subject: "Account Deletion Requested".to_string(),
    }, &template)
    .await
}

pub async fn send_confirm_email(to: String, params: IdentifierAndTokenParams) -> Result<()> {
    let template = ConfirmEmailTemplate {
        identifier: &params.identifier,
        token: &params.token,
    };
    send_template(MailOpts {
        to,
        subject: "Email Confirmation".to_string(),
    }, &template)
    .await
}

pub async fn send_update_email(to: String, params: IdentifierAndTokenParams) -> Result<()> {
    let template = UpdateEmailTemplate {
        identifier: &params.identifier,
        token: &params.token,
    };
    send_template(MailOpts {
        to,
        subject: "Email Update Requested".to_string(),
    }, &template)
    .await
}

// pub async fn send_plc_operation(to: String, params: IdentifierAndTokenParams) -> Result<()> {
//     let template = PLCUpdateTemplate {
//         identifier: &params.identifier,
//         token: &params.token,
//     };
//     send_template(MailOpts {
//         to,
//         subject: "PLC Update Operation Requested".to_string(),
//     }, &template)
//     .await
// }
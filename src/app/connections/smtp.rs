use crate::config::MailSmtpConfig;
use crate::LogService;
pub use lettre::error::Error as LettreError;
pub use lettre::message::header::ContentType as LettreContentType;
pub use lettre::message::Mailbox as LettreMailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::{Tls, TlsParameters};
pub use lettre::{
    Address as LettreAddress, Message as LettreMessage, SmtpTransport as LettreSmtpTransport,
    Transport as LettreTransport,
};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, Display, EnumString)]
pub enum SmtpConnectionError {
    ParsePortFail,
    MakeTlsParametersFail,
    EncryptionNotSupported,
}

pub fn get_smtp_transport(
    config: &MailSmtpConfig,
    log_service: &LogService,
) -> Result<LettreSmtpTransport, SmtpConnectionError> {
    log_service.info("Make smtp transport.");

    let host = config.host.to_owned();
    let port: u16 = config.port.to_owned().parse().map_err(|e| {
        log_service.error(format!("SmtpConnectionError::ParsePortFail - {:}", &e).as_str());
        SmtpConnectionError::ParsePortFail
    })?;
    let username = config.username.to_owned();
    let password = config.password.to_owned();

    let creds = Credentials::new(username, password);

    let mut mailer = LettreSmtpTransport::builder_dangerous(host.to_owned()).port(port);

    if config.encryption == "" {
    } else if config.encryption == "tls" {
        let tls_parameters = TlsParameters::new(host.into()).map_err(|e| {
            log_service
                .error(format!("SmtpConnectionError::MakeTlsParametersFail - {:}", &e).as_str());
            SmtpConnectionError::MakeTlsParametersFail
        })?;

        mailer = mailer.tls(Tls::Wrapper(tls_parameters));
    } else {
        log_service.error(
            format!(
                "SmtpConnectionError::EncryptionNotSupported - {}",
                &config.encryption
            )
            .as_str(),
        );
        return Err(SmtpConnectionError::EncryptionNotSupported);
    }

    let mailer = mailer.credentials(creds).build();

    Ok(mailer)
}

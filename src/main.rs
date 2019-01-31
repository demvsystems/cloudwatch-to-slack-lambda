use aws_lambda_events::event::sns::SnsEvent;
use dotenv::dotenv;
use lambda_runtime::{error::LambdaErrorExt, lambda, Context};
use std::env;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct CustomError {
    msg: String,
}

impl CustomError {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Self { msg: msg.into() }
    }
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for CustomError {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl LambdaErrorExt for CustomError {
    fn error_type(&self) -> &str {
        &self.msg
    }
}

fn get_log_level() -> log::Level {
    use std::env;
    match env::var("LOG_LEVEL")
        .unwrap_or_else(|_| "info".to_string())
        .as_ref()
    {
        "trace" => log::Level::Trace,
        "error" => log::Level::Error,
        "debug" => log::Level::Debug,
        "warn" => log::Level::Warn,
        _ => log::Level::Info,
    }
}

fn send_slack_msg(msg: &str) -> Result<(), CustomError> {
    use slack_hook::{PayloadBuilder, Slack};

    let webhook = env::var("SLACK_WEBHOOK").unwrap();
    let channel_name = env::var("CHANNEL_NAME").unwrap();
    let username = env::var("USERNAME").unwrap_or_else(|_| "SnsToSlackLambda".to_string());
    let slack = Slack::new(webhook.as_str()).unwrap();
    let p = PayloadBuilder::new()
        .text(msg)
        .channel(channel_name)
        .username(username)
        .icon_emoji(":bomb:")
        .build()
        .unwrap();

    match slack.send(&p) {
        Ok(()) => Ok(()),
        Err(x) => Err(CustomError::new(x.to_string())),
    }
}

fn handler(event: SnsEvent, _: Context) -> Result<(), CustomError> {
    use log::error;

    for record in event.records {
        if let Some(msg) = record.sns.message {
            if let Err(err) = send_slack_msg(&msg) {
                error!("{}", err);
                return Err(err);
            }
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    simple_logger::init_with_level(get_log_level()).unwrap();
    lambda!(handler);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::send_slack_msg;

    #[test]
    fn test_send_slack_msg() {
        assert!(send_slack_msg("Successfully executed Unit Test").is_ok());
    }
}

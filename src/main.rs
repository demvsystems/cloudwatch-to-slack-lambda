use aws_lambda_events::event::sns::SnsEvent;
use lambda_runtime::{error::LambdaErrorExt, lambda, Context};
use std::error::Error;
use std::fmt;
use std::result;

pub type BoxedResult<T> = result::Result<T, Box<Error>>;

#[derive(Debug, Clone)]
pub struct BoxedErr {
    msg: String,
}

impl BoxedErr {
    pub fn with<T, S: Into<String>>(msg: S) -> BoxedResult<T> {
        let e = Self { msg: msg.into() };

        Err(e.into())
    }
}

impl fmt::Display for BoxedErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for BoxedErr {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl LambdaErrorExt for BoxedErr {
    fn error_type(&self) -> &str {
        &self.msg
    }
}

fn send_slack_msg(msg: &str) -> BoxedResult<()> {
    use dotenv::dotenv;
    use slack_hook::{PayloadBuilder, Slack};
    use std::env;

    dotenv().ok();

    let webhook = env::var("SLACK_WEBHOOK").unwrap();
    let channel_name = env::var("CHANNEL_NAME").unwrap();
    let username = env::var("USERNAME").unwrap_or("SnsToSlackLambda".to_string());
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
        Err(x) => BoxedErr::with(x.to_string()),
    }
}

fn handler(event: SnsEvent, ctx: Context) -> BoxedResult<()> {
    for record in event.records {
        if let Some(msg) = record.sns.message {
            send_slack_msg(&msg)?;
        }
    }
    Ok(())
}

fn main() -> BoxedResult<()> {
    lambda!(handler);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::send_slack_msg;

    #[test]
    fn test_send_slack_msg() {
        send_slack_msg("Successfully executed Unit Test");
    }
}

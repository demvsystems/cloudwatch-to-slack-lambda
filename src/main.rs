use aws_lambda_events::event::cloudwatch_logs::{CloudwatchLogsData, CloudwatchLogsEvent};
use dotenv::dotenv;
use lambda_runtime::{error::LambdaErrorExt, lambda, Context};
use log::error;
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

fn  base64_decode_raw_log_to_gzip(data: &str) -> Result<Vec<u8>, CustomError> {
    use base64::decode;

    decode(data).map_err(|err| {
        error!("Couldn't base64 decode aws log data: {}", err);
        CustomError::new(err.to_string())
    })
}

fn gunzip_to_string(gzipped: Vec<u8>) -> Result<String, CustomError> {
    use std::io::Read;
    use flate2::read::GzDecoder;

    let mut raw_data = String::new();
    match GzDecoder::new(gzipped.as_slice()).read_to_string(&mut raw_data) {
        Ok(_) => Ok(raw_data),
        Err(err) => {
            error!("Couldn't gunzip decoded aws log data: {}", err);
            Err(CustomError::new(err.to_string()))
        }
    }
}

fn parse_string_to_logsdata(gunzipped: String) -> Result<CloudwatchLogsData, CustomError> {
    use serde_json::from_str;

    from_str(&gunzipped).map_err(|err| {
        error!(
            "Couldn't create CloudwatchLogsData from gunzipped json: {}",
            err,
        );
        CustomError::new(err.to_string())
    })
}

fn send_slack_msg_from_logsdata(logs_data: CloudwatchLogsData) -> Result<(), CustomError> {
    let msgs = logs_data
        .log_events
        .iter()
        .filter_map(|logs_log_event| logs_log_event.message.clone());
    for msg in msgs {
        if let Err(err) = send_slack_msg(&msg) {
            error!("{}", err);
            return Err(err);
        }
    }

    Ok(())
}

fn handler(event: CloudwatchLogsEvent, _: Context) -> Result<(), CustomError> {

    if let Some(data) = event.aws_logs.data {
        base64_decode_raw_log_to_gzip(&data)
            .and_then(gunzip_to_string)
            .and_then(parse_string_to_logsdata)
            .and_then(send_slack_msg_from_logsdata)?;
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

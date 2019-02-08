# CloudWatch to Slack Lambda

## Introduction
This lambda function is listening on a CloudWatch log and sends
matching log entries to a configured slack channel via webhooks.

## Build
You can build the lambda yourself. Have a look at [the official rust
lambda repository](https://github.com/awslabs/aws-lambda-rust-runtime) on how
to build it correctly.

However there is a Dockerfile provided with this repository to automate the
build. Switch into the cloned repository and execute:

```sh
docker build -t cloudwatch-to-slack-lambda .
docker run -v /tmp/artifacts:/export cloudwatch-to-slack-lambda
```

This will produce a `cloudwatch-to-slack-lambda.zip` at `/tmp/artifacts` on your
system, which can be uploaded to aws lambda.

## Configuration
The webhook, channel and username must be set via environment
variables. There is also a log level to be configured. See the
[.env.dist](https://github.com/demvsystems/cloudwatch-to-slack-lambda/blob/master/.env.dist)
for the exact keys.

#### SLACK_WEBHOOK
The URL of the webhook to be set. This is provided by Slack, after you setup an
app. The URL goes like this: https://hooks.slack.com/services/<random hash\>

#### CHANNEL_NAME
The channel where the log entries are posted to.

#### USERNAME
The username as whom the log entries are posted.

#### LOG_LEVEL
How many information should be logged. Valid values are:
- trace
- debug
- info
- warn
- error  
  
default: info

## Infrastructure
In this part we present a minimal infrastructure to use this lambda. The
infrastructure is presented as a terraform script.
```hcl
// Allow Lambda to assume a role
data "aws_iam_policy_document" "assume_by_lambda" {
  statement {
    sid     = "AllowAssumeByLambda"
    effect  = "Allow"
    actions = ["sts:AssumeRole"]

    principals {
      type        = "Service"
      identifiers = ["lambda.amazonaws.com"]
    }
  }
}

resource "aws_iam_role" "this" {
  name               = "SnsToSlackLambdaRole"
  assume_role_policy = "${data.aws_iam_policy_document.assume_by_lambda.json}"
}

// The Cloudwatch to Slack Lambda itself. Here the ZIP file is provided by the
// local machine. It can also be provided by other filesystems (like S3)
resource "aws_lambda_function" "this" {
  filename          = "cloudwatch-to-slack-lambda.zip"
  function_name     = "cloudwatch-to-slack"
  handler           = "cloudwatch-to-slack-lambda"
  runtime           = "provided"
  role              = "${data.aws_iam_role.this.arn}"

  environment {
    variables = {
      SLACK_WEBHOOK = "https://hooks.slack.com/services/<some hash>"
      CHANNEL_NAME  = "my_channel"
      USERNAME      = "cloudwatch-to-slack-lambda"
      LOG_LEVEL     = "error"
    }
  }
}

// Allow the Cloudwatch log to invoke the lambda function

resource "aws_lambda_permission" "this" {
  statement_id  = "AllowCloudwatchToSlackTrigger"
  action        = "lambda:InvokeFunction"
  function_name = "${aws_lambda_function.this.function_name}"
  principal     = "logs.eu-central-1.amazonaws.com" // differs between regions
}

resource "aws_cloudwatch_log_subscription_filter" "this" {
  name            = "cloudwatch-to-slack-logfilter"
  log_group_name  = "my_log_group" //The log group which should be subscribed to
  filter_pattern  = "ERROR" // Only passes log entries, which contain ERROR
  destination_arn = "${aws_lambda_function.this.arn}"
  depends_on      = ["aws_lambda_permission.this"]
}
```

This is a minimal working example to subscribe to a log group and pass entries
to the lambda, which match a specific pattern. The lambda sends those entries
to slack. Keep in mind that the lambda isn't capable of logging to cloudwatch
so far. Also note if you deploy your lambda into a VPC that the lambda needs
internet access to be able to send the messages to slack.

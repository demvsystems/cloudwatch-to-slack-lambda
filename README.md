# SNS to Slack Lambda

## Introduction
This lambda function is listening on a SNS topic and sends any message to a
configured slack channel via webhooks.

## Build
You can build the lambda yourself. Have a look at [the official rust
lambda repository](https://github.com/awslabs/aws-lambda-rust-runtime) on how
to build it correctly.

However there is a Dockerfile provided with this repository to automate the
build. Switch into the cloned repository and execute:

```sh
docker build -t sns-to-slack-lambda .
docker run -v /tmp/artifacts:/export sns-to-slack-lambda
```

This will produce a `sns-to-slack-lambda.zip` at `/tmp/artifacts` on your
system, which can be uploaded to aws lambda.

## Configuration
The webhook, channel and username must be set via environment
variables. There is also a log level to be configured. See the
[.env.dist](https://github.com/demvsystems/sns-to-slack-lambda/blob/master/.env.dist)
for the exact keys.

### SLACK_WEBHOOK
The URL of the webhook to be set. This is provided by Slack, after you setup an
app. The URL goes like this: https://hooks.slack.com/services/<random hash>

### CHANNEL_NAME
The channel where the SNS messages are posted to.

### USERNAME
The username as whom the SNS messages are posted.

### LOG_LEVEL
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

// A topic needs to be present as the lambda event source
resource "aws_sns_topic" "this" {
  name = "example-topic"
}

// The SNS to Slack Lambda itself. Here the ZIP file is provided by the local
// machine. It can also be provided by other filesystems (like S3)
resource "aws_lambda_function" "this" {
  filename          = "sns-to-slack-lambda.zip"
  function_name     = "sns-to-slack"
  handler           = "sns-to-slack-lambda"
  runtime           = "provided"
  role              = "${data.aws_iam_role.this.arn}"

  environment {
    variables = {
      SLACK_WEBHOOK = "https://hooks.slack.com/services/<some hash>"
      CHANNEL_NAME  = "my_channel"
      USERNAME      = "sns-to-slack-lambda"
      LOG_LEVEL     = "error"
    }
  }
}

// Allow the SNS topic to invoke the lambda function
resource "aws_lambda_permission" "this" {
  statement_id  = "AllowSNSToSlackLambdaExecutionFromSNS"
  action        = "lambda:invokeFunction"
  function_name = "${aws_lambda_function.this.function_name}"
  principal     = "sns.amazonaws.com"
  source_arn    = "${aws_sns_topic.this.arn}"
}
```

This is a minimal working example to send a message on the created SNS topic
and receive the message in slack through the lambda. Keep in mind that the
lambda isn't capable of logging to cloudwatch so far. Also note if you deploy
your lambda into a VPC that the lambda needs internet access to be able to send
the messages to slack.

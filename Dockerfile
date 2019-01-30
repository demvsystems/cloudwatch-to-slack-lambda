FROM ekidd/rust-musl-builder AS builder

ADD . .
RUN sudo chown -R rust:rust /home/rust
RUN cargo build --release
FROM alpine:latest AS package

ENV ARTIFACTS_DIR=/artifacts

RUN mkdir -p $ARTIFACTS_DIR

COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/sns-to-slack-lambda \
    $ARTIFACTS_DIR

WORKDIR $ARTIFACTS_DIR

RUN mv sns-to-slack-lambda bootstrap
RUN apk --no-cache add zip
RUN zip -r sns-to-slack-lambda.zip bootstrap && rm bootstrap

FROM package

ENV ARTIFACTS_DIR=/artifacts \
    EXPORT_DIR=/export

RUN mkdir -p $EXPORT_DIR

#Snapshot the directory
VOLUME $EXPORT_DIR

CMD find $ARTIFACTS_DIR -type f -name "sns-to-slack-lambda.zip" -exec cp '{}' $EXPORT_DIR \;

FROM rust:1.59 as builder
RUN mkdir -p /subvt/bin \
    && mkdir -p /subvt/src
WORKDIR /subvt/src
COPY ./.. ./
# add required nightly WASM target
RUN rustup default nightly \
    && rustup target add wasm32-unknown-unknown \
    && rustup default stable
# build SubVT backend
RUN cargo build --release
# copy executables
RUN cp target/release/subvt-app-service /subvt/bin/ \
  && cp target/release/subvt-block-processor /subvt/bin/ \
  && cp target/release/subvt-network-status-server /subvt/bin/ \
  && cp target/release/subvt-network-status-updater /subvt/bin/ \
  && cp target/release/subvt-notification-generator /subvt/bin/ \
  && cp target/release/subvt-notification-processor /subvt/bin/ \
  && cp target/release/subvt-onekv-updater /subvt/bin/ \
  && cp target/release/subvt-report-service /subvt/bin/ \
  && cp target/release/subvt-telegram-bot /subvt/bin/ \
  && cp target/release/subvt-telemetry-processor /subvt/bin/ \
  && cp target/release/subvt-validator-details-server /subvt/bin/ \
  && cp target/release/subvt-validator-list-server /subvt/bin/ \
  && cp target/release/subvt-validator-list-updater /subvt/bin/

FROM debian:buster-slim
# make bin directory
RUN mkdir -p /subvt/bin
# copy executables
COPY --from=builder /subvt/bin/ /subvt/bin/

FROM rust:1.58 as builder
RUN mkdir -p /subvt-backend/bin \
  && mkdir -p /subvt-backend/src
WORKDIR /subvt-backend/src
COPY ./.. ./
# add required nightly WASM target
RUN rustup default nightly \
    && rustup target add wasm32-unknown-unknown \
    && rustup default stable
# build SubVT backend
RUN cargo build --release
# copy executables
RUN cp target/release/subvt-app-service /subvt-backend/bin/ \
  && cp target/release/subvt-block-processor /subvt-backend/bin/ \
  && cp target/release/subvt-network-status-server /subvt-backend/bin/ \
  && cp target/release/subvt-network-status-updater /subvt-backend/bin/ \
  && cp target/release/subvt-notification-generator /subvt-backend/bin/ \
  && cp target/release/subvt-notification-sender /subvt-backend/bin/ \
  && cp target/release/subvt-onekv-updater /subvt-backend/bin/ \
  && cp target/release/subvt-report-service /subvt-backend/bin/ \
  && cp target/release/subvt-telemetry-processor /subvt-backend/bin/ \
  && cp target/release/subvt-validator-details-server /subvt-backend/bin/ \
  && cp target/release/subvt-validator-list-server /subvt-backend/bin/ \
  && cp target/release/subvt-validator-list-updater /subvt-backend/bin/

FROM debian:buster-slim
# make config and executable directories
RUN mkdir -p /subvt-backend/config \
  && mkdir -p /subvt-backend/bin
# copy config
COPY --from=builder /subvt-backend/src/subvt-config/config/ /subvt-backend/config/
# copy executables
COPY --from=builder /subvt-backend/bin/ /subvt-backend/bin/

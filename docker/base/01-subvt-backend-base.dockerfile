FROM rust:1.58 as builder
RUN mkdir -p /subvt-backend/src
RUN mkdir -p /subvt-backend/bin
WORKDIR /subvt-backend/src
COPY ./.. ./
# add required nightly WASM target
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown
RUN rustup default stable
RUN cargo build --release
# copy executables
RUN cp target/release/subvt-app-service /subvt-backend/bin/
RUN cp target/release/subvt-block-processor /subvt-backend/bin/
RUN cp target/release/subvt-network-status-server /subvt-backend/bin/
RUN cp target/release/subvt-network-status-updater /subvt-backend/bin/
RUN cp target/release/subvt-notification-generator /subvt-backend/bin/
RUN cp target/release/subvt-notification-sender /subvt-backend/bin/
RUN cp target/release/subvt-onekv-updater /subvt-backend/bin/
RUN cp target/release/subvt-report-service /subvt-backend/bin/
RUN cp target/release/subvt-telemetry-processor /subvt-backend/bin/
RUN cp target/release/subvt-validator-details-server /subvt-backend/bin/
RUN cp target/release/subvt-validator-list-server /subvt-backend/bin/
RUN cp target/release/subvt-validator-list-updater /subvt-backend/bin/

FROM debian:buster-slim
WORKDIR /subvt-backend/config
# install certificate authority certificates
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
RUN update-ca-certificates
# copy config
COPY --from=builder /subvt-backend/src/subvt-config/config/ ./
COPY --from=builder /subvt-backend/bin/subvt-app-service /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-block-processor /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-network-status-server /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-network-status-updater /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-notification-generator /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-notification-sender /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-onekv-updater /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-report-service /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-telemetry-processor /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-validator-details-server /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-validator-list-server /usr/local/bin/
COPY --from=builder /subvt-backend/bin/subvt-validator-list-updater /usr/local/bin/
FROM helikon/subvt-backend-base:0.1.2 as builder

FROM debian:buster-slim
# install certificate authority certificates, create config directory
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates \
    && mkdir -p /subvt-backend/config
# copy config files
COPY --from=builder /subvt-backend/config/ /subvt-backend/config/
# copy executable
COPY --from=builder /subvt-backend/bin/subvt-notification_generator /usr/local/bin/
CMD ["subvt-notification_generator"]
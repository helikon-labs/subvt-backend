FROM helikon/subvt-backend-base:0.1.2 as builder

FROM debian:buster-slim
# install certificate authority certificates, create config directory
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates \
    && mkdir -p /subvt/config
# copy config files
COPY ./_config/ /subvt/config/
# copy executable
COPY --from=builder /subvt/bin/subvt-notification-generator /usr/local/bin/
CMD ["subvt-notification-generator"]
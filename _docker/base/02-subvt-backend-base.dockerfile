FROM debian:bullseye-slim
# install certificate authority certificates, create config directory
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates \
    && mkdir -p /subvt/config
# copy config files and templates
COPY ./_config/ /subvt/config/
COPY ./_template/ /subvt/template/
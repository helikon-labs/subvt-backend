FROM debian:bullseye-slim
# install certificate authority certificates, create config directory
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates \
    && mkdir -p /subvt/config \
    && apt-get update \
    && apt-get -y install curl \
    && apt-get -y install fontconfig fonts-dejavu
# copy config files and templates
COPY ./_config/ /subvt/config/
COPY ./_template/ /subvt/template/
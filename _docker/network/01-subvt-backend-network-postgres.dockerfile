FROM timescale/timescaledb:2.5.2-pg14
ENV POSTGRES_PASSWORD postgres
ENV POSTGRES_HOST postgres
ENV PGDATA /var/lib/postgresql/data
# copy migration files
RUN mkdir -p /tmp/psql_data/
COPY ./_migrations/network/migrations/*.up.sql /tmp/psql_data/
COPY ./_docker/network/01-subvt-backend-network-postgres-init.sh /docker-entrypoint-initdb.d/
# install rust
#RUN apk add rustup
#RUN apk add build-base
#RUN rustup-init -y
#ENV PATH=#{user-home-abs-path}/.cargo/bin:"$PATH"
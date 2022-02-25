FROM postgres:14.2
ENV POSTGRES_PASSWORD postgres
ENV POSTGRES_HOST postgres
ENV PGDATA /var/lib/postgresql/data
# copy migration files
RUN mkdir -p /tmp/psql_data/
COPY ./_migrations/app/migrations/*.up.sql /tmp/psql_data/
COPY ./_docker/app/01-subvt-backend-app-postgres-init.sh /docker-entrypoint-initdb.d/
# install rust
#RUN apk add rustup
#RUN apk add build-base
#RUN rustup-init -y
#ENV PATH=#{user-home-abs-path}/.cargo/bin:"$PATH"
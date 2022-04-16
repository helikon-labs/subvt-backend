FROM postgres:14.2
ENV POSTGRES_PASSWORD postgres
ENV POSTGRES_HOST postgres
ENV PGDATA /var/lib/postgresql/data
# copy entry point
COPY ./_docker/base/03-subvt-postgres-entrypoint.sh /usr/local/bin/subvt-postgres-entrypoint.sh
RUN chmod +x /usr/local/bin/subvt-postgres-entrypoint.sh
# copy migration files
RUN mkdir -p /subvt/migrations/
COPY ./_migrations/app/migrations/*.up.sql /subvt/migrations/
# copy init script
COPY ./_docker/app/01-subvt-app-postgres-init.sh /docker-entrypoint-initdb.d/
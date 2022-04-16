#!/usr/bin/env bash
set -Eeo pipefail

# shellcheck disable=SC1090
source "$(which docker-entrypoint.sh)"

docker_setup_env
docker_create_db_directories

if [ -z "$DATABASE_ALREADY_EXISTS" ]; then
	docker_verify_minimum_env
	docker_init_database_dir
	pg_setup_hba_conf

	# only required for '--auth[-local]=md5' on POSTGRES_INITDB_ARGS
	export PGPASSWORD="${PGPASSWORD:-$POSTGRES_PASSWORD}"

	docker_temp_server_start "$@" -c max_locks_per_transaction=256
	docker_setup_db
else
	docker_temp_server_start "$@"
fi

# apply migrations regardless of whether a database exists of not
docker_process_init_files /docker-entrypoint-initdb.d/*
docker_temp_server_stop

exec postgres "$@"
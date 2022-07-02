#!/usr/bin/env bash
set -e

cd "${0%/*}" || exit # cd script directory
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "DROP DATABASE IF EXISTS subvt_app_test";
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "DROP DATABASE IF EXISTS subvt_network_test";
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -tc "SELECT 1 FROM pg_user WHERE usename = 'subvt'" | grep -q 1 ||  PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "CREATE USER subvt WITH ENCRYPTED PASSWORD 'subvt';"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "CREATE DATABASE subvt_app_test;"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE subvt_app_test TO subvt;"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "CREATE DATABASE subvt_network_test;"
PGPASSWORD=postgres psql -h 127.0.0.1 -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE subvt_network_test TO subvt;"
cd ../_migrations/app || exit
DATABASE_URL=postgres://subvt:subvt@127.0.0.1/subvt_app_test sqlx migrate run
cd ../network || exit
DATABASE_URL=postgres://subvt:subvt@127.0.0.1/subvt_network_test sqlx migrate run

#!/bin/bash

cd "$(dirname "$0")" || exit  # cd current directory
psql -c "DROP DATABASE IF EXISTS subvt_app_test";
psql -c "DROP DATABASE IF EXISTS subvt_kusama_test";
psql -tc "SELECT 1 FROM pg_user WHERE usename = 'subvt'" | grep -q 1 ||  psql -c "CREATE USER subvt WITH ENCRYPTED PASSWORD 'subvt';"
psql -c "CREATE DATABASE subvt_app_test;"
psql -c "GRANT ALL PRIVILEGES ON DATABASE subvt_app_test TO subvt;"
psql -c "CREATE DATABASE subvt_kusama_test;"
psql -c "GRANT ALL PRIVILEGES ON DATABASE subvt_kusama_test TO subvt;"
cd ../subvt-persistence/migrations/app || exit
DATABASE_URL=postgres://subvt:subvt@127.0.0.1/subvt_app_test sqlx migrate run
cd ../network || exit
DATABASE_URL=postgres://subvt:subvt@127.0.0.1/subvt_kusama_test sqlx migrate run
cargo test

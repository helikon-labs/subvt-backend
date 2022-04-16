echo "*********** START MIGRATION ***********"
# create subvt user if not exists
psql -U postgres -tc "SELECT 1 FROM pg_user WHERE usename = 'subvt'" | grep -q 1 || psql -U postgres -c "CREATE USER subvt WITH ENCRYPTED PASSWORD 'subvt';"
# create the app database if not exists
psql -U postgres -tc "SELECT 1 FROM pg_database WHERE datname = 'subvt_app'" | grep -q 1 || psql -U postgres -c "CREATE DATABASE subvt_app;"
psql -U postgres -c "ALTER DATABASE subvt_app OWNER TO subvt;"
# apply migrations
MIGRATION_FILES_DIR="/subvt/migrations"
for file in "$MIGRATION_FILES_DIR"/*.up.sql; do
    psql -U subvt -d subvt_app -f "${file}"
done
echo "************ END MIGRATION ************"
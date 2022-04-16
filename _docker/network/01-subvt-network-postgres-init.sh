echo "*********** START MIGRATION ***********"
# create subvt user if not exists
psql -U postgres -tc "SELECT 1 FROM pg_user WHERE usename = 'subvt'" | grep -q 1 || psql -U postgres -c "CREATE USER subvt WITH ENCRYPTED PASSWORD 'subvt';"
# create the network database if not exists
psql -U postgres -tc "SELECT 1 FROM pg_database WHERE datname = 'subvt_network'" | grep -q 1 || psql -U postgres -c "CREATE DATABASE subvt_network;"
psql -U postgres -c "ALTER DATABASE subvt_network OWNER TO subvt;"
# apply migrations
MIGRATION_FILES_DIR="/subvt/migrations"
for file in "$MIGRATION_FILES_DIR"/*.up.sql; do
    psql -U subvt -d subvt_network -f "${file}"
done
echo "************ END MIGRATION ************"
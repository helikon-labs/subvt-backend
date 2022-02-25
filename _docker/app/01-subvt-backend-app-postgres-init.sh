echo "*** INITIALIZING DATABASE ***"
psql -U postgres -c "CREATE USER subvt WITH ENCRYPTED PASSWORD 'subvt';"
psql -U postgres -c "CREATE DATABASE subvt_app;"
psql -U postgres -c "ALTER DATABASE subvt_app OWNER TO subvt;"
MIGRATION_FILES_DIR="/tmp/psql_data"
for file in "$MIGRATION_FILES_DIR"/*.sql; do
    psql -U subvt -d subvt_app -f "${file}"
done
echo "*** DATABASE INITIALIZED ***"
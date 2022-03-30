echo "*** INITIALIZING DATABASE ***"
psql -U postgres -c "CREATE USER subvt WITH ENCRYPTED PASSWORD 'subvt';"
psql -U postgres -c "CREATE DATABASE subvt_network;"
psql -U postgres -c "ALTER DATABASE subvt_network OWNER TO subvt;"
# psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE subvt_network TO subvt;"
MIGRATION_FILES_DIR="/tmp/psql_data"
for file in "$MIGRATION_FILES_DIR"/*.sql; do
    psql -U subvt -d subvt_network -f "${file}"
done
echo "*** DATABASE INITIALIZED ***"
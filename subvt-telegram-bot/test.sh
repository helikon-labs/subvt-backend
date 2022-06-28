#!/bin/bash
set -euo pipefail

function cleanup()
{
    echo "🧹 Clean up resources."
    echo "🔽 Stop and remove the app PostgreSQL container and volume."
    docker stop subvt_test_app_postgres &> /dev/null
    docker rm subvt_test_app_postgres &> /dev/null
    docker volume rm subvt_test_app_postgres_data &> /dev/null
    echo "🔽 Stop and remove the network PostgreSQL container and volume."
    docker stop subvt_test_network_postgres &> /dev/null
    docker rm subvt_test_network_postgres &> /dev/null
    docker volume rm subvt_test_network_postgres_data &> /dev/null
    echo "🏁 SubVT Telegram Bot testing completed."
}

if ! docker info > /dev/null 2>&1; then
  echo "🐳 This script uses docker, and it isn't running - please start docker and try again!"
  exit 1
fi
trap cleanup EXIT
echo "🏗 SubVT Telegram Bot setup started."
echo "🔼 Start the network PostgreSQL container and volume."
docker run --name subvt_test_network_postgres --platform linux/amd64 -d -p 15432:5432 -v subvt_test_network_postgres_data:/var/lib/postgresql/data helikon/subvt-network-postgres:latest &> /dev/null
echo "🔼 Start the app PostgreSQL container and volume."
docker run --name subvt_test_app_postgres --platform linux/amd64 -d -p 25432:5432 -v subvt_test_app_postgres_data:/var/lib/postgresql/data helikon/subvt-app-postgres:latest &> /dev/null
echo "😴 Sleep for 30 seconds until the database migrations are completed..."
sleep 30
echo "🟢 Start testing."
SUBVT_ENV=test cargo test -- --show-output
echo "✅ Testing completed successfully."
exit 0

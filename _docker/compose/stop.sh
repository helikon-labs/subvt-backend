#!/bin/bash

if ! docker info > /dev/null 2>&1; then
  echo "🐳 This script uses docker, and it isn't running - please start docker and try again!"
  exit 1
fi

docker-compose -p subvt_monitoring_services -f 07-docker-compose-subvt-monitoring.yml stop
docker-compose -p subvt_polkadot_services -f 06-docker-compose-subvt-polkadot-services.yml stop
docker-compose -p subvt_kusama_services -f 05-docker-compose-subvt-kusama-services.yml stop
docker-compose -p subvt_app_services -f 04-docker-compose-subvt-app-services.yml stop
docker-compose -p subvt_app_data_services -f 03-docker-compose-app-data.yml stop
docker-compose -p subvt_polkadot_data_services -f 02-docker-compose-polkadot-data.yml stop
docker-compose -p subvt_kusama_data_services -f 01-docker-compose-kusama-data.yml stop
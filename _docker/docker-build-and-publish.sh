#!/usr/bin/env bash
set -e
if [[ $1 == "" ]]
    then
    echo "Version parameter does not exist (eg 0.1.5)."
    exit 1
elif [[ $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Building and publishing SubVT Docker images v$1."
else
    echo "Invalid version parameter: $1. Enter a valid semver version (eg 0.1.5)."
    exit 1
fi

# cd to script directory
cd "${0%/*}" || exit

# backend base
docker build -t helikon/subvt-backend-base:"$1" -t helikon/subvt-backend-base:latest --no-cache --build-arg version="$1" -f ./base/02-subvt-backend-base.dockerfile ..
# backend lib
docker build -t helikon/subvt-backend-lib:"$1" -t helikon/subvt-backend-lib:latest --no-cache --build-arg version="$1" -f ./base/01-subvt-backend-lib.dockerfile ..

# app postgres
docker build -t helikon/subvt-app-postgres:"$1" -t helikon/subvt-app-postgres:latest --no-cache --build-arg version="$1" -f ./app/01-subvt-app-postgres.dockerfile ..
docker push --all-tags helikon/subvt-app-postgres
# app service
docker build -t helikon/subvt-app-service:"$1" -t helikon/subvt-app-service:latest --no-cache --build-arg version="$1" -f ./app/02-subvt-app-service.dockerfile ..
docker push --all-tags helikon/subvt-app-service
# notification processor
docker build -t helikon/subvt-notification-processor:"$1" -t helikon/subvt-notification-processor:latest --no-cache --build-arg version="$1" -f ./app/03-subvt-notification-processor.dockerfile ..
docker push --all-tags helikon/subvt-notification-processor

# network postgres
docker build -t helikon/subvt-network-postgres:"$1" -t helikon/subvt-network-postgres:latest --no-cache --build-arg version="$1" -f ./network/01-subvt-network-postgres.dockerfile ..
docker push --all-tags helikon/subvt-network-postgres
# block processor
docker build -t helikon/subvt-block-processor:"$1" -t helikon/subvt-block-processor:latest --no-cache --build-arg version="$1" -f ./network/02-subvt-block-processor.dockerfile ..
docker push --all-tags helikon/subvt-block-processor
# validator list updater
docker build -t helikon/subvt-validator-list-updater:"$1" -t helikon/subvt-validator-list-updater:latest --no-cache --build-arg version="$1" -f ./network/03-subvt-validator-list-updater.dockerfile ..
docker push --all-tags helikon/subvt-validator-list-updater
# active validator list server
docker build -t helikon/subvt-active-validator-list-server:"$1" -t helikon/subvt-active-validator-list-server:latest --no-cache --build-arg version="$1" -f ./network/04-subvt-active-validator-list-server.dockerfile ..
docker push --all-tags helikon/subvt-active-validator-list-server
# inactive validator list server
docker build -t helikon/subvt-inactive-validator-list-server:"$1" -t helikon/subvt-inactive-validator-list-server:latest --no-cache --build-arg version="$1" -f ./network/05-subvt-inactive-validator-list-server.dockerfile ..
docker push --all-tags helikon/subvt-inactive-validator-list-server
# validator details server
docker build -t helikon/subvt-validator-details-server:"$1" -t helikon/subvt-validator-details-server:latest --no-cache --build-arg version="$1" -f ./network/06-subvt-validator-details-server.dockerfile ..
docker push --all-tags helikon/subvt-validator-details-server
# network status updater
docker build -t helikon/subvt-network-status-updater:"$1" -t helikon/subvt-network-status-updater:latest --no-cache --build-arg version="$1" -f ./network/07-subvt-network-status-updater.dockerfile ..
docker push --all-tags helikon/subvt-network-status-updater
# network status server
docker build -t helikon/subvt-network-status-server:"$1" -t helikon/subvt-network-status-server:latest --no-cache --build-arg version="$1" -f ./network/08-subvt-network-status-server.dockerfile ..
docker push --all-tags helikon/subvt-network-status-server
# onekv updater
docker build -t helikon/subvt-onekv-updater:"$1" -t helikon/subvt-onekv-updater:latest --no-cache --build-arg version="$1" -f ./network/09-subvt-onekv-updater.dockerfile ..
docker push --all-tags helikon/subvt-onekv-updater
# telemetry processor
docker build -t helikon/subvt-telemetry-processor:"$1" -t helikon/subvt-telemetry-processor:latest --no-cache --build-arg version="$1" -f ./network/10-subvt-telemetry-processor.dockerfile ..
docker push --all-tags helikon/subvt-telemetry-processor
# telegram bot
docker build -t helikon/subvt-telegram-bot:"$1" -t helikon/subvt-telegram-bot:latest --no-cache --build-arg version="$1" -f ./network/11-subvt-telegram-bot.dockerfile ..
docker push --all-tags helikon/subvt-telegram-bot
# notification generator
docker build -t helikon/subvt-notification-generator:"$1" -t helikon/subvt-notification-generator:latest --no-cache --build-arg version="$1" -f ./network/12-subvt-notification-generator.dockerfile ..
docker push --all-tags helikon/subvt-notification-generator
# report service
docker build -t helikon/subvt-report-service:"$1" -t helikon/subvt-report-service:latest --no-cache --build-arg version="$1" -f ./network/13-subvt-report-service.dockerfile ..
docker push --all-tags helikon/subvt-report-service

# referendum updater
docker build -t helikon/subvt-referendum-updater:"$1" -t helikon/subvt-referendum-updater:latest --no-cache --build-arg version="$1" -f ./network/14-subvt-referendum-updater.dockerfile ..
docker push --all-tags helikon/subvt-referendum-updater
# kline updater
docker build -t helikon/subvt-kline-updater:"$1" -t helikon/subvt-kline-updater:latest --no-cache --build-arg version="$1" -f ./network/15-subvt-kline-updater.dockerfile ..
docker push --all-tags helikon/subvt-kline-updater
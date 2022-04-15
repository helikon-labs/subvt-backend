#!/bin/bash
if [[ $1 == "" ]]
    then
    echo "Version parameter does not exist (eg 0.1.5)."
    exit 1
elif [[ $1 =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Building and publishing the SubVT Docker images v$1."
else
    echo "Invalid version parameter: $1. Enter a valid semver version (eg 0.1.5)."
    exit 1
fi
cd "${0%/*}" || exit # cd script directory

# backend base
docker image rm helikon/subvt-backend-base:"$1"
docker build -t helikon/subvt-backend-base:"$1" --no-cache --build-arg version="$1" -f ../_docker/base/02-subvt-backend-base.dockerfile ..
# backend lib
docker image rm helikon/subvt-backend-lib:"$1"
docker build -t helikon/subvt-backend-lib:"$1" --no-cache --build-arg version="$1" -f ../_docker/base/01-subvt-backend-lib.dockerfile ..

# app postgres
docker image rm helikon/subvt-app-postgres:"$1"
docker build -t helikon/subvt-app-postgres:"$1" --no-cache --build-arg version="$1" -f ../_docker/app/01-subvt-app-postgres.dockerfile ..
docker push helikon/subvt-app-postgres:"$1"
# app service
docker image rm helikon/subvt-app-service:"$1"
docker build -t helikon/subvt-app-service:"$1" --no-cache --build-arg version="$1" -f ../_docker/app/02-subvt-app-service.dockerfile ..
docker push helikon/subvt-app-service:"$1"
# notification processor
docker image rm helikon/subvt-notification-processor:"$1"
docker build -t helikon/subvt-notification-processor:"$1" --no-cache --build-arg version="$1" -f ../_docker/app/03-subvt-notification-processor.dockerfile ..
docker push helikon/subvt-notification-processor:"$1"
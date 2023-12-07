#!/usr/bin/env bash
# Pushes docker images to container registry
set -eux -o pipefail

export BUILD_TAG="$(git describe --always --dirty)"

COMPOSE_DOCKER_CLI_BUILD=1 DOCKER_BUILDKIT=1 docker compose build --progress=plain

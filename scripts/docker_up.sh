#!/bin/bash

# Default to full docker-compose file
COMPOSE_FILE="docker/full.docker-compose.yaml"

# Parse command line arguments
while getopts "sf" opt; do
  case $opt in
    s)
      COMPOSE_FILE="docker/sil.docker-compose.yaml"
      ;;
    f)
      COMPOSE_FILE="docker/full.docker-compose.yaml"
      ;;
    \?)
      echo "Invalid option: -$OPTARG" >&2
      echo "Usage: $0 [-s] [-f]" >&2
      echo "  -s: Build SIL docker-compose" >&2
      echo "  -f: Build full docker-compose (default)" >&2
      exit 1
      ;;
  esac
done

docker-compose -f $COMPOSE_FILE up -d

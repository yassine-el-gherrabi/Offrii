#!/bin/sh
set -eu
echo "Running migrations..."
migrate
echo "Starting API server..."
exec rest-api

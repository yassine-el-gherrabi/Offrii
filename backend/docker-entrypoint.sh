#!/bin/sh
set -e
echo "Running migrations..."
migrate
echo "Starting API server..."
exec rest-api

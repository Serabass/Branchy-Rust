#!/bin/sh
set -e
export RESOLVER="${RESOLVER:-127.0.0.11}"
export BACKEND_HOST="${BACKEND_HOST:-web:3000}"
envsubst '${RESOLVER} ${BACKEND_HOST}' < /etc/nginx/conf.d/default.conf.template > /etc/nginx/conf.d/default.conf
exec nginx -g 'daemon off;'

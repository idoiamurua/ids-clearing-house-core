#########################################################################################
#
# Builds minimal runtime environment for the keyring-api
# Copyright 2019 Fraunhofer AISEC
#
#########################################################################################
FROM debian:stretch-slim

RUN apt-get update && \
    apt-get --no-install-recommends install -y ca-certificates gnupg2 libssl1.1 libc6 supervisor

RUN mkdir /server
WORKDIR /server

COPY target/release/keyring-api .
COPY keyring-api/supervisord.conf supervisord.conf

ENTRYPOINT ["/usr/bin/supervisord", "-c", "/server/supervisord.conf"]

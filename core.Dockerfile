#########################################################################################
#
# Builds minimal runtime environment for the Trackchain API
# Copyright 2019 Fraunhofer AISEC
#
#########################################################################################
FROM rustlang/rust:nightly AS build

# create server directories
RUN mkdir /server
RUN mkdir /server/core-lib
RUN mkdir /server/document-api
RUN mkdir /server/keyring-api

####### Prepare
WORKDIR /server
RUN mkdir core-lib/src && touch core-lib/src/lib.rs
RUN mkdir document-api/src && touch document-api/src/lib.rs
RUN mkdir keyring-api/src && touch keyring-api/src/lib.rs

# Toml 
ADD Cargo.toml .
WORKDIR /server/document-api
ADD core-lib/Cargo.toml /server/core-lib/.
ADD core-lib/src /server/core-lib/src
ADD document-api/Cargo.toml /server/document-api/.
ADD keyring-api/Cargo.toml /server/keyring-api/.

# Build dependencies
RUN cargo build --release

ENTRYPOINT ["/bin/bash"]

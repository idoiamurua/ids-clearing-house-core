# IDS Clearing House Core
The IDS Clearing House Core consists of two microservices that support the [Clearing House Service](https://github.com/Fraunhofer-AISEC/ids-clearing-house-service), which is a prototype implementation of the Clearing House component of the [Industrial Data Space](https://github.com/International-Data-Spaces-Association/IDS-G). The Clearing House provides an API to store and retrieve data. Data in the Clearing House is stored encrypted and practically immutable.

1. [Document API](document-api)
2. [Keyring API](keyring-api)

## Requirements
- [OpenSSL](https://www.openssl.org)
- [MongoDB](https://www.mongodb.com)
- ([Docker](https://www.docker.com))

## Configuration

### Document API
The Document API is responsible for storing the data and performs basic encryption and decryption for which it depends on the Keyring API. It is configured using the configuration file [`Rocket.toml`](document-api/Rocket.toml), which must specify a set of configuration options, such as the correct URLs of the database and other service apis:
- `daps_api_url`: Specifies the URL of the DAPS Service. Required to validate DAPS token
- `keyring_api_url`: Specifies the URL of the Keyring API
- `database_url`: Specifies the URL of the database to store the encrypted documents. Currently only mongodb is supported so URL is supposed to be `mongodb://<host>:<port>`
- `clear_db`: `true` or `false` indicates if the database should be cleared when starting the Service API or not. If `true` a restart will wipe the database! Starting the Service API on a clean database will initialize the database.

When starting the Clearing House Service API it also needs the following environment variables set:
- `API_LOG_LEVEL`: Allowed log levels are: `Off`, `Error`, `Warn`, `Info`, `Debug`, `Trace`

#### Example Configuration (docker-compose)
```
document-api:
    container_name: "document-api"
    depends_on:
        - keyring-api
        - document-mongo
    environment:
        # Allowed levels: Off, Error, Warn, Info, Debug, Trace
        - API_LOG_LEVEL=Info
    ports:
        - "8001:8001"
    volumes:
        - ./data/document-api/Rocket.toml:/server/Rocket.toml
        - ./data/certs:/server/certs
```


### Keyring API
The Keyring API is responsible for creating keys and the actual encryption and decryption of stored data. It is configured using the configuration file [`Rocket.toml`](keyring-api/Rocket.toml), which must specify a set of configuration options, such as the correct URLs of the database and other service apis:
- `daps_api_url`: Specifies the URL of the DAPS Service. Required to validate DAPS token
- `database_url`: Specifies the URL of the database to store document types and the master key. Currently only mongodb is supported so URL is supposed to be `mongodb://<host>:<port>`
- `clear_db`: `true` or `false` indicates if the database should be cleared when starting the Service API or not. If `true` a restart will wipe the database! Starting the Service API on a clean database will initialize the database.

When starting the Clearing House Service API it also needs the following environment variables set:
- `API_LOG_LEVEL`: Allowed log levels are: `Off`, `Error`, `Warn`, `Info`, `Debug`, `Trace`

The Keyring API requires that its database contains the acceptable document types. Currently only the IDS_MESSAGE type is supported and needs to be present in the database for the Keyring API to function properly. The database will be populated with an initial document type that needs to be located in `init_db/default_doc_type.json`.

#### Example Configuration (docker-compose)
```
keyring-api:
    container_name: "keyring-api"
    depends_on:
        - keyring-mongo
    environment:
        # Allowed levels: Off, Error, Warn, Info, Debug, Trace
        - API_LOG_LEVEL=Info
    ports:
        - "8002:8002"
    volumes:
        - ./data/keyring-api/init_db:/server/init_db
        - ./data/keyring-api/Rocket.toml:/server/Rocket.toml
        - ./data/certs:/server/certs
```

### DAPS
Both Document API and Keyring API need to be able to validate the certificate used by the DAPS. If the DAPS uses a self-signed certificate the certificate needs to be added in two places:
1. `/server/certs`: The microservice will load certificates in this folder in the container and use them for validation. The certificate needs to be in DER format.
2. `/usr/local/share/ca-certificates`: The microservice relies on openssl for parts of the validation and openssl will not trust a self-signed certificate unless it was added in this folder and `update-ca-certificates` was called in the docker container. Once this is done the container might need to be restarted.

If you are using [these dockerfiles](docker/) and use `daps.aisec.fraunhofer.de` as the DAPS, you only need to follow Step 1. The certificate needed for Step 1 can be found [here](document-api/certs).

## Docker Containers
Dockerfiles are located [here](docker/). There are two types of dockerfiles:
1. Simple builds (e.g. [dockerfile](docker/keyring-api.Dockerfile)) that require you to build the Service APIs yourself using [Rust](https://www.rust-lang.org)
2. Multistage builds (e.g. [dockerfile](docker/keyring-api-multistage.Dockerfile)) that have a stage for building the rust code

To build the containers check out the repository and in the main directory execute

`docker build -f docker/<dockerfile> . -t <image-name>`

Please read the Configuration section of the Service API you are trying to run, before using `docker run` oder `docker-compose`. All Containers build with the provided dockerfiles need two volumes:
1. The configuration file `Rocket.toml`is expected at `/server/Rocket.toml`
2. The folder containing the daps certificate is expected at `/server/certs`

Containers of the Keyring API require an additional volume:

3. `/server/init_db` needs to contain the `default_doc_type.json`

## Building from Source
The Clearing House Service APIs are written in [Rust](https://www.rust-lang.org) and can be build using

`cargo build --release`

The build requires OpenSSL to be installed.

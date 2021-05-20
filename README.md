# Core
Clearing House Core consists of the core-library and the document-api.

The core can be run standalone:
`docker-compose pull`
`docker-compose up`

The document-api is responsible for storing documents.

# Testing
To run only the unit tests:
`cargo test --lib`

To run only the integration tests:
`cargo test --test integration`


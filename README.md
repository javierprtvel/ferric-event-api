# ferric-event-api
A basic Event API made with Rust using Axum framework.

## Overview
This is a web service that implements an **Event API** with two endpoints: `/search` and `/ingest`.

The *search* endpoint returns a list of events occurring within a datetime range, specified using the `start_time` and `end_time` query parameters. It supports offset-based pagination.

```
GET http://localhost:8080/api/v1/search?start_time=2025-10-01T10:49:40Z&end_time=2025-12-25T00:00:00Z&limit=5&offset=20

Response Status: 200 OK
Response Body:
{
  "data": {
    "events": [
      {
        "id": "3fa85f64-5717-4562-b3fc-2c963f66afa6",
        "title": "Quevedo",
        "start_date": "2025-11-12",
        "start_time": "22:00:00",
        "end_date": "2025-11-12",
        "end_time": "23:00:00",
        "min_price": 15.99,
        "max_price": 39.99
      },
      {
        "id": "e762a900-93fc-4b71-bda9-ab81997ad262",
        "title": "Nirvana",
        "start_date": "2025-10-31",
        "start_time": "16:30:00",
        "end_date": "2025-10-31",
        "end_time": "23:59:59",
        "min_price": 75.0,
        "max_price": 99.99
      },
      {
        "id": "9765b4d4-ad7a-4672-a7a8-527bbec661b0",
        "title": "Tool",
        "start_date": "2025-12-24",
        "start_time": "21:00:00",
        "end_date": "2025-12-24",
        "end_time": "23:45:00",
        "min_price": 199.99,
        "max_price": 199.99
      }
    ]
  },
  "meta": {
    "limit": 5,
    "offset": 20
  },
  "error": null
}
```

The *ingest* endpoint triggers asynchronous event data ingestion, which updates the service's **event database** by retrieving data from external sources.
```
PATCH http://localhost:8080/api/v1/ingest

Response Status: 202 ACCEPTED
```

## Project Structure
The application structure follows a **hexagonal architecture** with *ports* and *adapters* and includes the typical layers of a **clean architecture**: *domain*, *application* and *infrastructure*.

Below is a summarized tree view of the project files::

```
.
├── Cargo.toml
├── compose.yml
├── Dockerfile
├── database
│   └── init
├── .sqlx
├── .env
├── src
│   ├── application
│   │   ├── ports
│   │   │   ├── provider.rs
|   |   |   └── repository.rs
│   ├── domain
│   ├── infrastructure
│   │   ├── adapters
│   │   │   ├── controller
│   │   │   │   ├── api
│   │   │   │   ├── handlers
│   │   │   ├── provider
│   │   │   └── repository
│   ├── lib.rs
│   └── main.rs
└── test
    ├── benchmark.yml
    ├── fixtures
    ├── run_benchmark.sh
    └── test_endpoints.py
```

- `Cargo.toml`: Rust project configuration and dependencies.
- `.sqlx/`: [`sqlx`](https://github.com/launchbadge/sqlx) metadata directory for offline query compilation checks.
- `.env`: environment variables serving as configuration to the application. Refer to `.env.example` for sample configuration values.
- `src/`:
  - `main.rs`: application entry point. It loads configuration, initializes tracing/logging and runs the server.
  - `domain/`: the domain model.
  - `application/`: the application layer containing the services (use cases).
  - `application/ports/`: interfaces for inbound/outbound interactions.
  - `infrastructure/`: the infrastructure layer containing the configuration, logging and server runtime logic.
  - `infrastructure/adapters/`: implementations for interfaces defined in `application/ports/`.
  - `infrastructure/adapters/controller/`: API routing, endpoint handlers and request/response shared context (Axum state).
- `test/`:
  - `test_endpoints.py`: Python script for simple endpoint testing.
  - `run_benchmark.sh`: script to run benchmarks using [`drill`](https://crates.io/crates/drill).
  - `benchmark.yml`: benchmark configuration.
  - `fixtures/`: test fixtures, such as third-party API XML responses and query parameters used in benchmarks.
- `Dockerfile`: Docker image definition using multi-stage build.
- `compose.yml`: Docker Compose configuration for easy local testing.
- `database/init/`: database initialization scripts for the PostgreSQL docker container used in `compose.yml`.

## Run
You can start the service easily by launching the **event database** Docker container and running the application with an unoptimized *debug* build:

```
docker compose up -d event-db
cargo run
```

Then you can call the API via `curl` commands, for example:

```
curl -X GET http://localhost:8080/api/v1/search?start_time=2025-10-01T10:49:40Z&end_time=2025-12-25T00:00:00Z&limit=5&offset=20
```

Alternatively, you can start the service inside a container with an optimized *release* build by running the services configured in `compose.yml` using Docker Compose. This way you have all the components from the project managed as a Docker Compose application.

```
docker compose up -d
curl -X GET http://localhost:8080/api/v1/search?start_time=2025-10-01T10:49:40Z&end_time=2025-12-25T00:00:00Z&limit=5&offset=20
```

## Test
Run all unit and integration tests in the Rust source code:

```
cargo test
```

Test API endpoints by running the application and then executing the Python script:

```
python test/test_endpoints.py
```

Test the service performance by running the benchmark:

```
./test/run_benchmark.sh
# or
bash test/run_benchmark.sh
```
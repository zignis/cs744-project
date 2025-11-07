# cs744-project

A key-value HTTP server based on PostgreSQL with in-memory caching.

## Running with Docker

```shell
cd server
docker-compose up --build
```

By default, the HTTP server is exposed at http://localhost:8000

Update environment variables in `server/docker-compose.yaml` if required.

## Usage

### Create/update a key-value pair

```shell
curl -X POST \
     -H "Content-Type: application/json" \
     -d '{"key": "example_key_1", "value": "example_value_1"}' \
     -i \
      http://localhost:8000
```

### Fetch a key-value pair

```shell
curl -X GET -i http://localhost:8000/<key>
```

### Delete a key-value pair

```shell
curl -X DELETE -i http://localhost:8000/<key>
```

## Testing

Run automated tests inside `server/` using `cargo test`.
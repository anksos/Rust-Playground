# Running the docker postgres sql

```shell
docker run --name rust-postgres-db -e POSTGRES_PASSWORD=password -e POSTGRES_USER=postgres -e POSTGRES_DB=rust-axum-rest-api -p 5432:5432 -d postgres
# in case of persistent data
#   -v pgdata:/var/lib/postgresql/data \
#   -d postgres
```

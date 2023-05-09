# NephroFlow Development Environment Manager

Switch NephroFlow development environments easily.

## Dump and restore databases

Include `export PGPASSWORD=...` in your `.zshrc` or `.bashrc`. This way,
you don't have to enter the postgres password for every postgres action.

### Dump

1. Start the postgres service (with exposed ports): `docker-compose run --service-ports postgres`
2. `nfde database dump $name`

### Restore

1. Start the postgres service (with exposed ports): `docker-compose run --service-ports postgres`
2. Make sure nothing is using the database (the tool will only try to kill
   the `rails` process of the `web` service)
3. `nfde database restore` will open a fuzzy picker of your saved database

## Save and load docker images

### Save

```bash
nfde docker save $name
```

### Load

```bash
nfde docker load
```

## Prerequisites

- cargo (install [rustup](https://www.rust-lang.org/tools/install))
- docker
- `brew install postgresql`
  - drop_db
  - create_db
  - pg_dump
  - pg_restore
- (connection to the postgres database from localhost)

## Config

```bash
nfde config
```

### Docker image, container and database names

- api_container_name (default: "nephroflow-web-1")
- api_image_name (default: "nephroflow/server")
- nephroflow_database_name (default: "nephroflow_development")

### Backup paths

- backup_image_path
- backup_database_path

## Binaries

### nfde

```bash
nfde -h
```

### run_api

```bash
run_api -h
```

## Installation

Install cargo and run

```bash
cargo install --path .
```

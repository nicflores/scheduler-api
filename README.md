# Scheduler API

### Setup Instructions

Run the following commands to setup sqlx:

```bash
docker-compose up -d
sqlx database create
sqlx migrate run
cargo sqlx prepare
```

The `.env` file contains the `DATABASE_URL` variable which is used by sqlx to connect to the database. The docker-compose.yml file create a postgres database
with the same parameters as the ones in the `.env` file.

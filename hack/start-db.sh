if ! [ -x "$(command -v psql)" ]; then
    echo >&2 "Error: psql is not installed."
    exit 1
fi

if ! [ -x "$(command -v sqlx)" ]; then
    echo >&2 "Error: sqlx is not installed."
    echo >&2 "Use:"
    echo >&2 "\tcargo install sqlx-cli --no-default-features --features rustls,postgres"
    echo >&2 "to install it."

    exit 1
fi

ENV_FILE="$(dirname $0)/../.env"

if ! [ -f "$ENV_FILE" ]; then
    echo >&2 "Error: no .env file found."
    echo >&2 "Please, copy the .env.example file to .env"
    echo >&2 "\tcp .env.example .env"

    exit 1
fi

source $(dirname $0)/../.env

if ! [ -z "${SKIP_DOCKER}" ]; then

    if [ -x "$(command -v podman)" ]; then
        podman compose up -d
    elif [ -x "$(command -v docker)" ]; then
        docker compose up -d
    else
        echo >&2 "Docker or Podman are not installed."
        exit 1
    fi

    export PGPASSWORD="${POSTGRES_PASSWORD}"
    until psql -h "${POSTGRES_HOST}" -U "${POSTGRES_USER}" -p "${POSTGRES_PORT}" -d "postgres" -c '\q'; do
        >&2 echo "Postgres is still unavailable - sleeping"
        sleep 1
    done
fi

export DATABASE_URL
sqlx database create
sqlx migrate run

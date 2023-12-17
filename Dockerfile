FROM rust:1.72-slim-bookworm AS build
WORKDIR /app
RUN apt-get update && apt-get install clang=1:14.0-55.7~deb12u1 mold=1.10.1+dfsg-1 -y --no-install-recommends
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends ca-certificates \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/zero2prod zero2prod
COPY config config
ENV APP_ENV prod

ENTRYPOINT ["./zero2prod"]


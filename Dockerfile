FROM rust:1.88-alpine3.22 AS build

RUN apk update
RUN apk add libpq-dev musl-dev

WORKDIR /lonklink

COPY Cargo.lock Cargo.toml ./

# Separate stage for building dependencies, to better utilize Docker's caching
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch
RUN cargo build --locked --profile release
RUN rm target/release/LonkLink target/release/LonkLink.d
RUN rm -rf src/ target/release/deps/LonkLink*

COPY askama.toml ./
COPY src/ ./src/
COPY migrations/ ./migrations/
COPY .sqlx ./.sqlx/

ENV SQLX_OFFLINE=true
RUN cargo build --frozen --profile release

FROM alpine:3.22 AS runner

RUN apk update
RUN apk add libpq

WORKDIR /lonklink

COPY --from=build /lonklink/target/release/LonkLink .

ENTRYPOINT ./LonkLink migrate_db && ./LonkLink app
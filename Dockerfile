FROM rust:1.85.0-alpine3.21 AS build

RUN set -ex && \
    apk add --no-progress --no-cache \
        musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock /app/
COPY ./src /app/src
COPY ./.sqlx /app/.sqlx
RUN cargo build --release

FROM alpine:3.21 AS run

RUN apk add --no-progress --no-cache tzdata

ENV UID=65532
ENV GID=65532
ENV USER=nonroot
ENV GROUP=nonroot

RUN addgroup -g $GID $GROUP && \
    adduser --shell /sbin/nologin --disabled-password \
    --no-create-home --uid $UID --ingroup $GROUP $USER

WORKDIR /app/
COPY --from=build /app/target/release/mdr-service ./
COPY ./app.yaml ./
USER $USER

ENTRYPOINT ["/app/mdr-service"]

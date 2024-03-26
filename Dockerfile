FROM rust:1.76.0-alpine3.19 as builder
WORKDIR /app
ARG TARGETARCH ARG TARGETVARIANT
RUN apk add --no-cache alpine-sdk musl-dev libressl-dev pkgconf

COPY . .

RUN cargo build --release

FROM alpine:3.19 as final
COPY --from=builder /app/target/release/ceph-rbd-metrics /bin/ceph-rbd-metrics
CMD ["/bin/ceph-rbd-metrics"]

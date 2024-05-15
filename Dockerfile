FROM public.ecr.aws/docker/library/rust:latest AS chef

RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
USER nonroot
COPY --chown=nonroot:nonroot --from=builder /app/target/release/rust-scheduler-service /user/local/bin/
EXPOSE 3000

ENTRYPOINT ["/user/local/bin/rust-scheduler-service"]

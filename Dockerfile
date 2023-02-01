FROM rust:1.66 as builder
COPY . .
RUN cargo build --package thank_you_rocket_rs --release

# probably move compilation to another install of debian:buster-slim, compile, then send over to the running image.

FROM debian:buster-slim
COPY --from=builder /target/release/thank_you_rocket_rs ./target/release/thank_you_rocket_rs
EXPOSE 8000
CMD ["./target/release/thank_you_rocket_rs"]

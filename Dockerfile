FROM rust:1.66 as builder
COPY . .
RUN rustup target add wasm32-unknown-unknown
RUN cargo build --package thank_you_rocket_rs --release
RUN cargo install --locked trunk
RUN chmod +x setup_sub_projects.sh
RUN ./setup_sub_projects.sh

FROM debian:buster-slim
COPY --from=builder /target/release/thank_you_rocket_rs ./target/release/thank_you_rocket_rs
COPY --from=builder /discreet_math_fib_dist ./discreet_math_fib_dist
COPY --from=builder /discreet_math_fib_dist ./target/release/discreet_math_fib_dist
COPY --from=builder /rhythm_rs_dist ./target/release/rhythm_rs_dist
COPY --from=builder /rhythm_rs_dist ./rhythm_rs_dist
EXPOSE 80
CMD ["./target/release/thank_you_rocket_rs"]

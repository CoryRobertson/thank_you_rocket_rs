FROM rust:1.66 as builder
COPY . .
# install the wasm target, needed for building rhythm and fib
RUN rustup target add wasm32-unknown-unknown
# build the project
RUN cargo build --package thank_you_rocket_rs --release
# install trunk to this builder, needed to build sub projects
RUN cargo install --locked trunk
# mark the setup script as runnable
RUN chmod +x setup_sub_projects.sh
# run the setup sub projects
RUN ./setup_sub_projects.sh

FROM debian:bookworm
# copy built binary from builder to debian image
COPY --from=builder /target/release/thank_you_rocket_rs ./thank_you_rocket_rs
# copy distribution version of fib project into image
COPY --from=builder /discreet_math_fib_dist ./discreet_math_fib_dist
# copy rhythm_rs dist version from builder into image
COPY --from=builder /rhythm_rs_dist ./rhythm_rs_dist
# copy static dir from builder into image
COPY --from=builder /static ./static
# copy rocket toml from builder into image.
COPY --from=builder /Rocket.toml .

EXPOSE 80
VOLUME ["/output"]
CMD ["./thank_you_rocket_rs"]

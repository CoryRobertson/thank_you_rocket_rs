FROM debian:bookworm
# this is a docker image that allows the use of pre built stuff from this project, much faster for testing smaller changes.
COPY ./target/release/thank_you_rocket_rs .
COPY ./rhythm_rs_dist ./rhythm_rs_dist
COPY ./discreet_math_fib_dist ./discreet_math_fib_dist
COPY ./Rocket.toml .
COPY ./static ./static
EXPOSE 80
VOLUME ["/output"]
CMD ["./thank_you_rocket_rs"]

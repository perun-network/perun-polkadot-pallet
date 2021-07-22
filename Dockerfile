FROM rust:slim-buster

WORKDIR /pallet
COPY . .
RUN cargo test --no-run --all-features

CMD [ "/usr/local/cargo/bin/cargo", "test", "--all-features" ]

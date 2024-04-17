FROM rust:1.70-bookworm as build

# create a new empty shell project
RUN USER=root cargo new --bin ctf
WORKDIR /ctf

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./src ./src

# build for release
RUN rm ./target/release/deps/ctf*
RUN cargo build --release

# our final base
FROM debian:bookworm-slim

# create working directory
WORKDIR /ctf

# install dependencies needed for app
RUN apt-get update && apt-get install -y sqlite3 openssl && rm -rf /var/lib/apt/lists/* 

# copy the build artifact from the build stage
COPY --from=build /ctf/target/release/ctf .

# copy relevant resouce files from local repo
COPY ./static ./static/
COPY ./templates ./templates/

# open up port by default
EXPOSE 3000

# set the startup command to run your binary
CMD ["./ctf"]

FROM rust:1.77 as build

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
FROM rust:1.77

# create working directory
WORKDIR /ctf

# copy the build artifact from the build stage
COPY --from=build /ctf/target/release/ctf .

# copy relevant resouce files from local repo
COPY ./static ./static/
COPY ./templates ./templates/

# set the startup command to run your binary
CMD ["./ctf"]

FROM node:14
COPY ./.git /build/.git
COPY ./secretnote-fe /build/secretnote-fe
WORKDIR /build/secretnote-fe
RUN npm install && npm run build

#FROM rustembedded/cross:x86_64-unknown-linux-musl
FROM rust:latest
RUN apt-get update && apt-get install -y musl-tools && apt-get clean
WORKDIR /build
RUN rustup target add x86_64-unknown-linux-musl
COPY ./Cargo.* /build/
COPY ./src /build/src
# RUN cargo install --target x86_64-unknown-linux-musl --path . && \
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=0 /build/fe /fe
COPY --from=1 /build/target/x86_64-unknown-linux-musl/release/secretnote /secretnote
CMD ["/secretnote"]

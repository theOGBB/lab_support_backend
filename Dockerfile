FROM rust
WORKDIR /
COPY . .
RUN cargo build
EXPOSE 8080
CMD ["cargo", "run"]
FROM rust
WORKDIR /
COPY . .
RUN cargo build
EXPOSE 7878
CMD ["cargo", "run"]
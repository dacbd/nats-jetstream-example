FROM rust:1.80

WORKDIR /app

COPY . .

RUN cargo build

ENTRYPOINT ["cargo"]
CMD ["run"]


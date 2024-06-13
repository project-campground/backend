FROM golang:latest

# Copy the files
WORKDIR /app
ADD . .

# Dependencies
RUN apt update && \
    apt install --no-install-recommends -y \
        git

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup default stable && \
    export RUSTFLAGS='--cfg surrealdb_unstable' && \
    cargo install surrealdb-migrations

VOLUME /tmp

# Run
EXPOSE 1313
CMD ["go", "run", "."]
FROM rust:1.67 as builder
WORKDIR /usr/src/minato-lite
COPY . .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates tzdata && rm -rf /var/lib/apt/lists/*
ARG APP=/app
RUN mkdir -p ${APP}
WORKDIR ${APP}
COPY --from=builder /usr/local/cargo/bin/minato-lite /app/minato-lite

ENV TZ=Etc/UTC
EXPOSE 8080
CMD [ "/app/minato-lite" ]
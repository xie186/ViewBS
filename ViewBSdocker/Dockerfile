FROM rust:1-bookworm AS builder

WORKDIR /src
COPY . .
RUN cargo build --release --all-features

FROM debian:bookworm-slim

LABEL software="ViewBS" \
      container="ViewBS" \
      about.summary="ViewBS - a pure Rust toolkit for bisulfite sequencing methylation analysis" \
      about.home="https://github.com/xie186/ViewBS" \
      software.version="0.2.0-alpha.0" \
      about.license="GPL-3.0-only" \
      about.tags="biology::nucleic-acids,field::biology,field::biology:bioinformatics,interface::commandline,role::program,use::analysing"

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /src/target/release/ViewBS /usr/local/bin/ViewBS

WORKDIR /data
ENTRYPOINT ["ViewBS"]
CMD ["--help"]

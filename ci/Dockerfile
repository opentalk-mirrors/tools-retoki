# SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

FROM git.opentalk.dev:5050/opentalk/backend/containers/rust:1.81.0-bookworm AS builder

WORKDIR /relbo

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY src src

RUN RUSTFLAGS=-Ctarget-feature=-crt-static cargo auditable build --release

FROM debian:bookworm-slim

RUN apt update \
    && apt install -y git \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /relbo/target/release/relbo /usr/opt

ENTRYPOINT [ "/opt/relbo" ]

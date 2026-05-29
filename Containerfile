# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
# SPDX-License-Identifier: EUPL-1.2

FROM debian:trixie-slim

COPY target/release/retoki /usr/local/bin/retoki
RUN retoki --version
RUN apt-get --update install --assume-yes git jq just

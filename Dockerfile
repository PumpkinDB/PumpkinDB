# Copyright (c) 2017, All Contributors (see CONTRIBUTORS file)
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.

# Parts of this file are based on docker-rustup by Hannes de Jager
#
# The MIT License (MIT)
#
# Copyright (c) 2016 Hannes de Jager
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:

# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.

# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

FROM ubuntu:16.04

RUN apt-get update \
    && apt-get install -y curl file sudo build-essential

RUN curl https://sh.rustup.rs > sh.rustup.rs \
    && sh sh.rustup.rs -y \
    && . $HOME/.cargo/env \
    && echo 'source $HOME/.cargo/env' >> $HOME/.bashrc \
    && rustup update \
    && rustup target add x86_64-unknown-linux-musl

RUN . $HOME/.cargo/env && mkdir /pumpkindb && cd /pumpkindb && rustup override set nightly

COPY Makefile /pumpkindb/
COPY Cargo.* /pumpkindb/
COPY pumpkindb_engine /pumpkindb/pumpkindb_engine
COPY pumpkindb_server /pumpkindb/pumpkindb_server
COPY pumpkindb_mio_server /pumpkindb/pumpkindb_mio_server
COPY pumpkindb_term /pumpkindb/pumpkindb_term
COPY pumpkindb_client /pumpkindb/pumpkindb_client
COPY pumpkinscript /pumpkindb/pumpkinscript
COPY tests /pumpkindb/tests
COPY doc /pumpkindb/doc

RUN . $HOME/.cargo/env && cd /pumpkindb && cargo build --all --release
RUN    mv /pumpkindb/target/release/pumpkindb /usr/local/bin \
    && mv /pumpkindb/target/release/pumpkindb-term /usr/local/bin

RUN echo "[storage]\npath=\"/db\"" > /pumpkindb/pumpkindb.toml

EXPOSE 9981

VOLUME /db
WORKDIR /pumpkindb

CMD pumpkindb -c /pumpkindb/pumpkindb.toml
FROM rust:1.70 as base

FROM base as install-snap
RUN apt-get update && apt-get install -y git curl binutils clang make
RUN git clone https://github.com/Homebrew/brew ~/.linuxbrew/Homebrew \
        && mkdir ~/.linuxbrew/bin \
        && ln -s ../Homebrew/bin/brew ~/.linuxbrew/bin \
        && eval $(~/.linuxbrew/bin/brew shellenv) \
        && brew --version \
        && brew tap aws/tap && brew install aws-sam-cli \
        && sam --version
ENV PATH=~/.linuxbrew/bin:~/.linuxbrew/sbin:$PATH

FROM install-snap as install-zig
RUN brew install zig

FROM install-zig as install-cargolambda
RUN cargo install --locked cargo-lambda

FROM install-cargolambda as builder
WORKDIR /app
COPY . .
RUN cargo lambda build

CMD ["cargo", "lambda", "watch"]
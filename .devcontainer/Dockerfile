FROM rust:1.80-bookworm

RUN rustup component add rustfmt

## Setup test coverage report tool
RUN rustup component add llvm-tools-preview
RUN export RUSTFLAGS="-Cinstrument-coverage"
RUN export LLVM_PROFILE_FILE="optimisation-server-%p-%m.profraw"

# grcov . -s . --binary-path ./target/debug/ -t html --branch --ignore-not-existing -o ./target/debug/coverage/

## Installing Toolset
RUN apt update -y
RUN apt install git llvm-dev libclang-dev clang autoconf autotools-dev g++ gcc build-essential cpputest automake autoconf libtool pkgconf cmake -y

ENV BRANCH_NAME=""

ADD ./../.. /app

EXPOSE 8000
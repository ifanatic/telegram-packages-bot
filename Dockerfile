FROM ifanatic/rustssl
MAINTAINER Nikolay Oshnurov "onlk@yandex.ru"
ADD . /rust
RUN cargo build --verbose

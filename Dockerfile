FROM ifanatic/rustssl
MAINTAINER Nikolay Oshnurov "onlk@yandex.ru"

ADD ./target/release/packagesbot /opt/packagesbot

ENTRYPOINT /opt/packagesbot

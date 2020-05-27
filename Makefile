PKG_CONFIG_PATH := ${PWD}/pkgconfig

all:
	cargo +stable-gnu build --release
	mkdir -p include
	cbindgen --quiet --output ./include/husk.h
	PKG_CONFIG_PATH=${PKG_CONFIG_PATH} \
	c-for-go -ccincl -ccdefs husk.yml

sample: all
	PKG_CONFIG_PATH=${PKG_CONFIG_PATH} \
	go get -v .

VERSION:=2.6.0
SHASUM:=5936bf5e9943128b53a9e534e0a6842532b52bb5

.PHONY: all

all: LICENSE.txt duk_config.h duktape.h duktape.c

LICENSE.txt: duktape-${VERSION}.tar.xz
	tar xvf $^ --strip-components 1 'duktape-${VERSION}/$@'
	touch $@

duk_config.h duktape.h duktape.c: duktape-${VERSION}.tar.xz
	tar xvf $^ --strip-components 2 'duktape-${VERSION}/src/$@'
	touch $@

duktape-${VERSION}.tar.xz:
	curl -sSfOL https://github.com/svaarala/duktape/releases/download/v${VERSION}/duktape-${VERSION}.tar.xz
	printf "%s  %s" "${SHASUM}" $@ | shasum -c -

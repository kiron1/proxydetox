
VERSION:=2.5.0
SHASUM:=d91e1982f707cb0b7be42f1f11cf00e972465319

.PHONY: all

all: LICENSE.txt duk_config.h duktape.h duktape.c

LICENSE.txt: duktape-${VERSION}.tar.xz
	tar xvf $^ --strip-components 1 'duktape-${VERSION}/$@'
	touch $@

duk_config.h duktape.h duktape.c: duktape-${VERSION}.tar.xz
	tar xvf $^ --strip-components 2 'duktape-${VERSION}/src/duk*.[ch]'
	touch $@

duktape-${VERSION}.tar.xz:
	curl -OL# https://github.com/svaarala/duktape/releases/download/v${VERSION}/duktape-${VERSION}.tar.xz
	printf "%s  %s" "${SHASUM}" $@ | shasum -c -

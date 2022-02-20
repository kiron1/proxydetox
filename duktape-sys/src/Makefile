
VERSION:=2.7.0
SHASUM:=8ff5465c9c335ea08ebb0d4a06569c991b9dc4661b63e10da6b123b882e7375e82291d6b883c2644902d68071a29ccc880dae8229447cebe710c910b54496c1d

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

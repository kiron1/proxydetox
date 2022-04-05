
DOCSRC:=$(shell find doc/ -type f)
PUBLICDIR:=public
BOOKDIR:=${PUBLICDIR}/book
WWWSRC:=$(shell find www/ -type f)

all:
	cargo build --release

pages: ${BOOKDIR}/index.html $(patsubst www/%,${PUBLICDIR}/%,${WWWSRC})
	@true

clean:
	-cargo clean
	-rm -rf -- "${PUBLICDIR}"

${BOOKDIR}/index.html: ${DOCSRC} book.toml
	@mkdir -p "${BOOKDIR}"
	mdbook build -d "${BOOKDIR}"

${PUBLICDIR}/%: www/%
	@install -v -m 0644 "$^" "$@"

.PHONY: all pages clean

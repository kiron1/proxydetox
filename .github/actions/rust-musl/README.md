# Rust musl build container

## Build container

```sh
docker build -t proxydetox-builder .
```

## Run container

```sh
docker run -v $(pwd):/github --rm proxydetox-builder
```

Interactive:

```sh
docker run -v $(pwd):/github --rm -it --entrypoint bash proxydetox-builder
```

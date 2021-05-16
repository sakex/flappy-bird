# Flappy Bird

## Run the local server

```
cd web-app
cargo run
```

Visit http://localhost:8080

## Build WASM and copy target to the correct path

```
wasm-pack build --target web && cp -r pkg web-app                                                                                                                                                                                 ─╯
```

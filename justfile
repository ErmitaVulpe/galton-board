build:
    cargo b -r

build-wasm:
    cp -r assets/ web/assets
    cargo build --target wasm32-unknown-unknown -r
    wasm-bindgen --no-typescript --target web \
        --out-dir ./web/ \
        --out-name "galton-board" \
        ./target/wasm32-unknown-unknown/release/galton-board.wasm
    wasm-opt -Oz --enable-bulk-memory-opt --strip-debug --all-features -o web/galton-board_bg.wasm web/galton-board_bg.wasm

build-wasm-dev:
    cp -r assets/ web/assets
    cargo build --target wasm32-unknown-unknown
    wasm-bindgen --no-typescript --target web \
        --out-dir ./web/ \
        --out-name "galton-board" \
        ./target/wasm32-unknown-unknown/debug/galton-board.wasm

dev:
    cargo r -F bevy/dynamic_linking

clean:
    cargo clean

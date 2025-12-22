

CRATE_NAME="automata-web"
TARGET_NAME="automata-web"
OUT_FILE_NAME="./root/automata.wasm"
mkdir root
TARGET="../target"


cargo build --package automata-web --target wasm32-unknown-unknown --release
TARGET_NAME="${CRATE_NAME}.wasm"
WASM_PATH="${TARGET}/wasm32-unknown-unknown/release/$TARGET_NAME"

wasm-bindgen ${WASM_PATH} --out-dir root --out-name ${OUT_FILE_NAME} --no-modules --no-typescript
wasm-opt ${OUT_FILE_NAME} -O2 --fast-math -g -o ${OUT_FILE_NAME}
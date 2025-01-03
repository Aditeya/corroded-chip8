run path:
	cargo run -- {{path}}

run-wasm:
	wasm-pack build --target web wasm/ 
	mv wasm/pkg/wasm_bg.wasm web/
	mv wasm/pkg/wasm.js web/
	miniserve web/

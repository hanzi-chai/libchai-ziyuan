c:
	cargo run --release -- encode

o:
	cargo run --release -- optimize

p:
	cargo run --release -- optimize -t 10

f:
	sudo cargo flamegraph --profile benchmark -- optimize

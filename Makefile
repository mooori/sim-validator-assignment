run:
	cargo run -p sim-validator-assignment -- \
		run \
		--num-blocks 1000 \
		--num-shards 1 \
		--seats-per-shard 250 \
		--stake-per-seat 1 \
		--max-malicious-stake-per-shard 1/2

download:
	cargo run -p sim-validator-assignment -- \
		download \
		--protocol near	\
		--rpc-url 'https://rpc.testnet.near.org' \
		--out ./validator_data.json

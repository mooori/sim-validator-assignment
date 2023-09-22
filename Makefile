run:
	cargo run -p sim-validator-assignment -- \
		--num-blocks 1000 \
		--num-shards 1 \
		--seats-per-shard 250 \
		--stake-per-seat 1 \
		--max-malicious-stake-per-shard 1/2
		
run:
	cargo run -- \
		--num-blocks 1000 \
		--num-shards 4 \
		--seats-per-shard 64 \
		--stake-per-seat 100 \
		--max-malicious-stake-per-shard 2/3
		
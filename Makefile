.PHONY: run
run:
	cargo run -p sim-validator-assignment -- \
		run \
		--num-blocks 1000 \
		--num-shards 4 \
		--seats-per-shard 250 \
		--stake-per-seat 1 \
		--max-malicious-stake-per-shard 1/2 \
		--include-partial-seats

.PHONY: download
download:
	cargo run -p sim-validator-assignment -- \
		download \
		--protocol near	\
		--rpc-url 'https://archival-rpc.testnet.near.org' \
		--block-height 140053158 \
		--out ./validator_data.json

# Runs a simulation reading data from `./validator_data.json` which is assumed to be obtained by
# invoking the `download` target defined above.
.PHONY: run-with
run-with:
	cargo run -p sim-validator-assignment -- \
		run \
		--num-blocks 1000 \
		--num-shards 4 \
		--seats-per-shard 250 \
		--stake-per-seat 50000000000000000000000000000 \
		--max-malicious-stake-per-shard 2/3 \
		--include-partial-seats \
		--validator-data ./validator_data.json

# Runs a simulation reading data from `./validator_data.json` which is assumed to be obtained by
# invoking the `download` target defined above.
.PHONY: seat-stats
seat-stats:
	cargo run -p sim-validator-assignment -- \
		seat-stats \
		--validator-data ./validator_data.json \
		--stake-per-seat 500000000000000000000000000000 \
		--include-partial-seats

cargo build --release
chain-spec-builder create --relay-chain "dev" --para-id 1000 --runtime \
                target/release/wbuild/mock-runtime/mock_runtime.wasm named-preset development
# target/release/minimal-template-node --chain ./chain_spec.json --dev --tmp --log info

# Check if executable exists, if not build it
if [ ! -f "target/release/minimal-template-node" ]; then
    echo "Executable not found. Building..."
    cargo build --release --workspace
fi

parallel --ungroup --no-notice ::: \
      'target/release/minimal-template-node --chain ./chain_spec.json --dev --tmp --log info --alice --listen-addr /ip4/0.0.0.0/tcp/53102/ws --discover-local --node-key c12b6d18942f5ee8528c8e2baf4e147b5c5c18710926ea492d09cbd9f6c9f82a' \
      'target/release/minimal-template-node --chain ./chain_spec.json --dev --tmp --log info --bob --listen-addr /ip4/0.0.0.0/tcp/54102/ws --discover-local --bootnodes /ip4/127.0.0.1/tcp/53102/ws/p2p/12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2 --unsafe-force-node-key-generation'
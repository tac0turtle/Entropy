#!/bin/bash

set -e

os_type=""

if [[ $OSTYPE == "linux-gnu"* ]]; then
    os_type="linux"
elif [[ $OSTYPE == "darwin"* ]]; then
    os_type="darwin"
else
    echo "$OSTYPE is not supported."
    exit 1
fi

docker-compose up -d
if ! hash solana 2>/dev/null; then
    echo Installing Solana tool suite ...
    curl -sSf https://raw.githubusercontent.com/solana-labs/solana/v1.3.9/install/solana-install-init.sh | sh -s - v1.3.9
    export PATH="/home/runner/.local/share/solana/install/active_release/bin:$PATH"
    echo Generating keypair ...
    solana-keygen new -o ~/.config/solana/id.json --no-passphrase --silent
fi
solana config set --url "http://localhost:8899"

mkdir -p target/dex
curl -s -L "https://github.com/serum-community/serum-dex/releases/download/v2/serum_dex-$os_type.so" > ./target/dex/serum_dex.so
sleep 1
solana airdrop 10000
DEX_PROGRAM_ID="$(solana deploy --use-deprecated-loader serum_dex.so | jq .programId -r)"
echo DEX_PROGRAM_ID: "$DEX_PROGRAM_ID"
curl -s -L "https://github.com/serum-community/serum-dex/releases/download/v2/crank-$os_type" > ./target/dex/crank
chmod +x ./target/dex/crank
./target/dex/crank l pyserum-setup ~/.config/solana/id.json "$DEX_PROGRAM_ID"

echo "dex_program_id: $DEX_PROGRAM_ID" >> ./target/dex/crank.log
cat ./target/dex/crank.log

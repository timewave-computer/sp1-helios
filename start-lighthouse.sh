docker run \
  -p 127.0.0.1:5052:5052 \
  -v $HOME/.lighthouse:/root/.lighthouse \
  sigp/lighthouse \
  lighthouse \
    beacon \
    --network sepolia \
    --checkpoint-sync-url https://sepolia.checkpoint.sigp.io \
    --http \
    --http-address 0.0.0.0 \
    --disable-deposit-contract-sync


cargo build --release

ssh "$MFRO_DEPLOY_HOST" killall "publish"
sleep 1

scp "./target/release/publish" "$MFRO_DEPLOY_HOST:server/publish/"
ssh "$MFRO_DEPLOY_HOST" "startup/publish.sh"

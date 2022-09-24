yarn build
scp main.js api.mfro.me:server/publish/main.js
ssh api.mfro.me startup/publish.sh

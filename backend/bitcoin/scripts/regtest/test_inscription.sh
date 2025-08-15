# example inscription
docker cp inscription.txt ord:/inscription.txt

./scripts/ord_wallet.sh inscribe --file ./inscription.txt --fee-rate 1

# check transaction
# docker compose exec bitcoind bitcoin-cli -regtest getrawtransaction <tx> true

# mine
docker compose exec bitcoind bitcoin-cli -regtest -rpcwallet=default getnewaddress
docker compose exec bitcoind bitcoin-cli -regtest -rpcwallet=default generatetoaddress 1 <output-address>

# should see the inscription
./scripts/ord-wallet.sh wallet --server-url http://0.0.0.0:8080 inscriptions

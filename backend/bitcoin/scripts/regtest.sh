./bin/bitcoind -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data --port=18444

./bin/bitcoin-cli -conf=$(pwd)/bitcoin.conf -regtest createwallet "testwallet"

./bin/bitcoin-cli -conf=$(pwd)/bitcoin.conf -regtest loadwallet "testwallet"

./bin/bitcoin-cli -conf=$(pwd)/bitcoin.conf -regtest getbalance

./bin/bitcoin-cli -conf=$(pwd)/bitcoin.conf -regtest getnewaddress

./bin/bitcoin-cli -conf=$(pwd)/bitcoin.conf -regtest generatetoaddress 101 "$(./bin/bitcoin-cli -conf=$(pwd)/bitcoin.conf -regtest getnewaddress)"

./bin/bitcoin-cli -conf=$(pwd)/bitcoin.conf -regtest -named sendtoaddress "mfYar2FgNtTyZ4hstJcDReQnbUoRhshWKA" 10000 fee_rate=25

./bin/bitcoin-cli -conf=$(pwd)/bitcoin.conf -datadir=$(pwd)/data stop
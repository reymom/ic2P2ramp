# evm collect fees from vault
dfx canister call backend withdraw_evm_fees '(11155111 : nat64, 109985601535148549124 : nat, opt "0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF")'
dfx canister call backend withdraw_evm_fees '(11155111 : nat64, 7160497397524000 : nat, null)'

dfx canister call backend withdraw_evm_fees '(84532 : nat64, 669659 : nat, opt "0x036CbD53842c5426634e7929541eC2318f3dCF7e")'
dfx canister call backend withdraw_evm_fees '(84532 : nat64, 349000000000000 : nat, null)'

dfx canister call backend withdraw_evm_fees '(11155420 : nat64, 590200058320000 : nat, null, opt 20000)'

dfx canister call backend withdraw_evm_fees '(421614 : nat64, 935839 : nat, opt "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d")'
dfx canister call backend withdraw_evm_fees '(421614 : nat64, 175954720000000 : nat, null)'

# for chain_id; for token[chain_id];
dfx canister call backend transfer_evm_funds '(11155111 : nat64, "0xReceiverAddress", 109985601535148549124 : nat, opt "0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF", opt 100000)'
dfx canister call backend transfer_evm_funds '(11155111 : nat64, "0xReceiverAddress", 50000000000000000 : nat, null, opt 20000)'

dfx canister call backend transfer_evm_funds '(84532 : nat64, "0xReceiverAddress", 669659 : nat, opt "0x036CbD53842c5426634e7929541eC2318f3dCF7e", opt 25000)'
dfx canister call backend transfer_evm_funds '(84532 : nat64, "0xReceiverAddress", 49887249045132304 : nat, null, opt 20000)'

dfx canister call backend transfer_evm_funds '(11155420 : nat64, "0xReceiverAddress", 9999990000000000 : nat, null, opt 20000)'

dfx canister call backend transfer_evm_funds '(421614 : nat64, "0xReceiverAddress", 935839 : nat, opt "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d", opt 100000)'
dfx canister call backend transfer_evm_funds '(421614 : nat64, "0xReceiverAddress", 10553818100000000 : nat, null, opt 80000)'

# for ledger_canisters;
dfx canister call backend transfer_canister_funds '(principal "ryjl3-tyaaa-aaaaa-aaaba-cai", principal "", 1270000 : nat)'
dfx canister call backend transfer_canister_funds '(principal "mc6ru-gyaaa-aaaar-qaaaq-cai", principal "", 270 : nat)'

curl -o download_latest_icp_ledger.sh "https://raw.githubusercontent.com/dfinity/ic/00a4ab409e6236d4082cee4a47544a2d87b7190d/rs/rosetta-api/scripts/download_latest_icp_ledger.sh"
chmod +x download_latest_icp_ledger.sh
./download_latest_icp_ledger.sh

# add this in the dfx.json file
# "candid": icp_ledger.did,
# "wasm" : icp_ledger.wasm.gz,

export MINTER_PRINCIPAL=$(dfx identity get-principal --identity minter)
export DEFAULT_ACCOUNT_PRINCIPAL=$(dfx identity get-principal --identity default)

dfx deploy icp_ledger_canister --argument "(
  variant {
    Init = record {
      minting_account = record {
        owner = principal \"$MINTER_PRINCIPAL\";
        subaccount = null;
      };
      fee_collector_account = null;
      transfer_fee = 10_000:nat;
      decimals = null;
      max_memo_length = null;
      token_symbol = \"LICP\";
      token_name = \"Local ICP\";
      metadata = vec {};
      initial_balances = vec {
        record {
          record {
            owner = principal \"$DEFAULT_ACCOUNT_PRINCIPAL\";
            subaccount = null;
          };
          10_000_000_000:nat;
        };
      };
      feature_flags = null;
      maximum_number_of_accounts = null;
      accounts_overflow_trim_quantity = null;
      archive_options = record {
        num_blocks_to_archive = 1000:nat64;
        max_transactions_per_response = null;
        trigger_threshold = 1000:nat64;
        max_message_size_bytes = null;
        cycles_for_archive_creation = null;
        node_max_memory_size_bytes = null;
        controller_id = principal \"$DEFAULT_ACCOUNT_PRINCIPAL\";
        more_controller_ids = null;
      };
    }
  }
)"

dfx deploy ckbtc_ledger_canister_testnet --argument "(
  variant {
    Init = record {
      minting_account = record { 
        owner = principal \"ml52i-qqaaa-aaaar-qaaba-cai\";
        subaccount = null;
      };
      transfer_fee = 10:nat;
      token_symbol = \"ckTESTBTC\";
      token_name = \"Chain key testnet Bitcoin\";
      metadata = vec {};
      initial_balances = vec {
        record {
          record {
            owner = principal \"$DEFAULT_ACCOUNT_PRINCIPAL\";
            subaccount = null;
          };
          10_000_000_000:nat;
        };
      };
      max_memo_length = opt 80;
      archive_options = record {
        num_blocks_to_archive = 1000;
        trigger_threshold = 2000;
        max_message_size_bytes = null;
        cycles_for_archive_creation = opt 1_000_000_000_000;
        node_max_memory_size_bytes = opt 3_221_225_472;
        controller_id = principal \"$DEFAULT_ACCOUNT_PRINCIPAL\";
      };
    }
  }
)"
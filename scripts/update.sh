# Update with no changes
dfx deploy backend --argument "( variant { Upgrade = null } )"

# Update with new ecdsa key
dfx deploy backend --upgrade-unchanged --argument "(
  variant { 
    Upgrade = opt record {
      ecdsa_key_id = opt record {
      name = \"test_key_1\";
        curve = variant { secp256k1 };
      };
      chains = null;
      paypal = null;
      revolut = null;
      proxy_url = null;
    }
  }
)"

# Modify a chain
dfx deploy backend --upgrade-unchanged --argument "(
  variant {
    Upgrade = opt record {
      ecdsa_key_id = null;
      chains = opt vec {
        record {
          chain_id = 11155111 : nat64;
          vault_manager_address = \"${CONTRACT_BASE_SEPOLIA}\";
          services = variant { EthSepolia = opt vec { variant { Alchemy } } };
          currency_symbol = \"ETH\";
        }
      };
      paypal = null;
      revolut = null;
      proxy_url = null;
    }
  }
)"

# Deploy a new chain
dfx deploy backend --upgrade-unchanged --argument "(
  variant {
  Upgrade = opt record {
    ecdsa_key_id = null;
    chains = opt vec {
      record {
        chain_id = 12345678 : nat64;
        vault_manager_address = \"${CONTRACT_BASE_SEPOLIA}\";
        services = variant {
          Custom = record {
            chainId = 12345678 : nat64;
            services = vec {
              record { url = \"https://base-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
            };
          }
        };
        currency_symbol = \"ETH\";
      }
    };
    paypal = null;
    revolut = null;
    proxy_url = null;
  }
)"

# Change paypal config
dfx deploy backend --upgrade-unchanged --argument "(
  variant {
  Upgrade = opt record {
    ecdsa_key_id = null;
    chains = null;
    paypal = opt record {
      client_id = \"new_paypal_client_id\";
      client_secret = \"new_paypal_client_secret\";
      api_url = \"new_paypal_api_url\";
    };
    revolut = null;
    proxy_url = null;
  }
)"

# Change revolut config
dfx deploy backend --upgrade-unchanged --argument "(
  variant {
  Upgrade = opt record {
    ecdsa_key_id = null;
    chains = null;
    paypal = null;
    revolut = opt record {
      client_id = \"new_revolut_client_id\";
      api_url = \"new_revolut_api_url\";
      proxy_url = \"new_proxy_url\";
      private_key_der = blob \"$(echo $(cat revolut_certs/private.key | base64 -w 0) | base64 --decode)\";
      kid = \"new_kid\";
      tan = \"new_tan\";
    };
    proxy_url = null;
  }
)"

# Change proxy url
dfx deploy backend --upgrade-unchanged --argument "(
  variant {
    Upgrade = opt record {
      ecdsa_key_id = null;
      chains = null;
      paypal = null;
      revolut = null;
      proxy_url = opt \"testing\";
    }
  }
)" --ic
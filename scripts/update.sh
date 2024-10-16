# Update with no changes
dfx deploy backend --argument "( variant { Upgrade = null } )" --upgrade-unchanged

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
dfx deploy backend_prod --upgrade-unchanged --argument "(
  variant {
    Upgrade = opt record {
      ecdsa_key_id = null;
      chains = opt vec {
        record {
          chain_id = 1 : nat64;
          vault_manager_address = \"${CONTRACT_MAINNET}\";
          services = variant { EthMainnet = opt vec { variant { Alchemy } } };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 8453 : nat64;
          vault_manager_address = \"${CONTRACT_BASE}\";
          services = variant {
            Custom = record {
              chainId = 8453 : nat64;
              services = vec {
                record { url = \"https://base-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 10 : nat64;
          vault_manager_address = \"${CONTRACT_OP}\";
          services = variant {
            Custom = record {
              chainId = 10 : nat64;
              services = vec {
                record { url = \"https://opt-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
      };
      paypal = null;
      revolut = null;
      proxy_url = null;
    }
  }
)" --ic

dfx deploy backend --upgrade-unchanged --argument "(
  variant {
    Upgrade = opt record {
      ecdsa_key_id = null;
      chains = opt vec {
        record {
          chain_id = 84532 : nat64;
          vault_manager_address = \"${CONTRACT_BASE_SEPOLIA}\";
          services = variant {
            Custom = record {
              chainId = 84532 : nat64;
              services = vec {
                record { url = \"https://base-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = opt vec {
                  record { name = \"Cache-Control\"; value = \"max-age=60\" };
                } };
                record { url = \"https://base-sepolia.infura.io/v3/${INFURA_API_KEY}\"; headers = opt vec {
                  record { name = \"Cache-Control\"; value = \"max-age=60\" };
                } };
                record { url = \"https://rpc.ankr.com/base_sepolia/${ANKR_PROJECT}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 11155420 : nat64;
          vault_manager_address = \"${CONTRACT_OP_SEPOLIA}\";
          services = variant {
            Custom = record {
              chainId = 11155420 : nat64;
              services = vec {
                record { url = \"https://opt-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = opt vec {
                  record { name = \"Cache-Control\"; value = \"max-age=60\" };
                } };
                record { url = \"https://optimism-sepolia.infura.io/v3/${INFURA_API_KEY}\"; headers = opt vec {
                  record { name = \"Cache-Control\"; value = \"max-age=60\" };
                } };
                record { url = \"https://rpc.ankr.com/optimism_sepolia/${ANKR_PROJECT}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
      };
      paypal = null;
      revolut = null;
      proxy_url = null;
    }
  }
)" --ic

dfx deploy backend --upgrade-unchanged --argument "(
  variant {
    Upgrade = opt record {
      ecdsa_key_id = null;
      chains = opt vec {
        record {
          chain_id = 84532 : nat64;
          vault_manager_address = \"${CONTRACT_BASE_SEPOLIA}\";
          services = variant {
            Custom = record {
              chainId = 84532 : nat64;
              services = vec {
                record { url = \"https://rpc.ankr.com/base_sepolia/${ANKR_PROJECT}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
      };
      paypal = null;
      revolut = null;
      proxy_url = null;
    }
  }
)" --ic

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
          client_id = \"${PAYPAL_CLIENT_ID}\";
          client_secret = \"${PAYPAL_CLIENT_SECRET}\";
          api_url = \"api-m.paypal.com\";
        };
        revolut = null;
        proxy_url = null;
    }
  }
)" --ic

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
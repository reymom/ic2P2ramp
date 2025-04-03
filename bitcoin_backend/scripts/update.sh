dfx deploy bitcoin_backend --argument "( variant { Upgrade = null } )" --upgrade-unchanged --ic

dfx deploy bitcoin_backend --upgrade-unchanged --argument "(
  variant {
    Upgrade = opt record {
        network = null;
        btc_principal = null;
        unisat = opt record {
          api_url = \"open-api-testnet4.unisat.io\";
          api_key = \"${UNISAT_API_KEY}\";
        };
        proxy_url = null;
    }
  }
)" --ic

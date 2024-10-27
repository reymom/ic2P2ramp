dfx generate backend

dfx deploy backend --argument "(
  variant {
    Reinstall = record {
      ecdsa_key_id = record {
        name = \"dfx_test_key\";
        curve = variant { secp256k1 };
      };
      chains = vec {};
      paypal = record {
        client_id = \"${PAYPAL_CLIENT_ID}\";
        client_secret = \"${PAYPAL_CLIENT_SECRET}\";
        api_url = \"https://api-m.sandbox.paypal.com\";
      };
      revolut = record {
        client_id = \"${REVOLUT_CLIENT_ID}\";
        api_url = \"https://sandbox-oba.revolut.com\";
        proxy_url = \"some\";
        private_key_der = blob \"$(echo $(cat revolut_certs/private.key | base64 -w 0) | base64 --decode)\";
        kid = \"kid_0\";
        tan = \"test-jwk.s3.eu-west-3.amazonaws.com\";
      };
      truelayer = record {
        client_id = \"${TRUELAYER_CLIENT_ID}\";
        client_secret = \"${TRUELAYER_CLIENT_SECRET}\";
        host_url = \"truelayer-sandbox.com\";
        proxy_url = \"https://truelayer.ic2p2ramp.xyz:9443\";
        private_key = blob \"$(echo $(cat certs/truelayer_ec512-private-key.pem | base64 -w 0) | base64 --decode)\";
        kid = \"${TRUELAYER_KID}\";
      };
      proxy_url = \"https://ic2p2ramp.xyz\";
    }
  }
)"
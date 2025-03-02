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
      proxy_url = \"https://dc55-92-178-206-241.ngrok-free.app\";
      private_key_der = blob \"$(echo $(cat revolut_certs/private.key | base64 -w 0) | base64 --decode)\";
      kid = \"kid_0\";
      tan = \"test-jwk.s3.eu-west-3.amazonaws.com\";
    };
    proxy_url = \"https://ic2p2ramp.xyz\";
  }

  }
)"
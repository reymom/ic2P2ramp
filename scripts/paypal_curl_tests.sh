curl -v -X POST "https://api-m.sandbox.paypal.com/v1/oauth2/token"\
 -u "CLIENT_ID:CLIENT_SECRET"\
 -H "Content-Type: application/x-www-form-urlencoded"\
 -d "grant_type=client_credentials"

curl -v -X GET https://api-m.sandbox.paypal.com/v2/checkout/orders/4UC03319AV493141A \
-H 'Authorization: Bearer ${ACCESS_TOKEN}' 
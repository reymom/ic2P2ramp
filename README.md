# IC2P2Ramp

IC2P2Ramp is a decentralized protocol combining the Internet Computer (ICP) with Ethereum and other EVM blockchains to facilitate onramping and offramping of digital assets. The protocol utilizes multiple canisters for various functionalities, including HTTPS outcalls, EVM RPC communication, and exchange rate retrieval. It features a backend and a frontend for interacting with the protocol.

## Deployments

### Canisters on ICP

- **HTTPS Outcalls Canister**: Facilitates secure HTTP(S) requests from ICP canisters.
- **EVM RPC Canister**: An ICP canister smart contract for communicating with Ethereum and other EVM blockchains using an on-chain API.
- **Exchange Rate Canister**: Retrieves and provides exchange rates for various assets.
- **Backend Canister**: Handles business logic, including order management and communication with other canisters.
- **Frontend Canister**: Provides the user interface for interacting with the IC2P2Ramp protocol.

## Canisters

### HTTPS Outcalls Canister

The HTTPS Outcalls Canister enables secure HTTP(S) requests from ICP canisters, allowing for external data fetching and API interactions.

### EVM RPC Canister

The EVM RPC Canister is a smart contract on the ICP that communicates with Ethereum and other EVM blockchains. It provides an on-chain API for interacting with smart contracts and retrieving blockchain data.

### Exchange Rate Canister

The Exchange Rate Canister retrieves and provides exchange rates for various assets. It uses an external API to fetch real-time exchange rates and serves this data to other canisters within the protocol.

### Backend Canister

The Backend Canister handles the core business logic of the IC2P2Ramp protocol. It manages orders, communicates with the EVM RPC canister for blockchain interactions, and verifies payments using the HTTPS Outcalls canister.

### Frontend Canister

The Frontend Canister provides a user-friendly interface for interacting with the IC2P2Ramp protocol. Users can create and manage orders, view exchange rates, and perform onramping and offramping operations.

## Usage

### Build

To build the canisters, use the following command:

```shell
dfx build
```

## Interact

- Call the `get_usd_exchange_rate` method to retrieve the exchange rate for a given asset:

```shell
dfx canister call backend get_usd_exchange_rate '( "ETH" )'
```

- Retrieve and verify a paypal order using the backend canister:

```sh
dfx canister call backend verify_transaction '( "0", transaction_id = "4UC03319AV493141A" )'
```

### Locally:

Run the following commands in a new, empty project directory:

```sh
git clone https://github.com/fxgst/evm-rpc-rust.git # Download this starter project
cd evm-rpc-rust # Navigate to the project directory
dfx start --clean --background # Run dfx in the background
npm install # Install project dependencies
npm run setup # Install packages, deploy canisters, and generate type bindings

npm start # Start the development server
```

## 🚀 Develop

The frontend will update automatically as you save changes.
For the backend, run `dfx deploy backend` to redeploy.
To redeploy all canisters (front- and backend), run `dfx deploy`.

When ready, run `dfx deploy --network ic` to deploy your application to the ICP mainnet.

## 🛠️ Technology Stack

- [Vite](https://vitejs.dev/): high-performance tooling for front-end web development
- [React](https://reactjs.org/): a component-based UI library
- [TypeScript](https://www.typescriptlang.org/): JavaScript extended with syntax for types
- [Sass](https://sass-lang.com/): an extended syntax for CSS stylesheets
- [Prettier](https://prettier.io/): code formatting for a wide range of supported languages
- [Rust CDK](https://docs.rs/ic-cdk/): the Canister Development Kit for Rust
- [EVM RPC canister](https://github.com/internet-computer-protocol/evm-rpc-canister): call Ethereum RPC methods from the Internet Computer

## 📚 Documentation

- [Internet Computer docs](https://internetcomputer.org/docs/current/developer-docs/ic-overview)
- [Internet Computer wiki](https://wiki.internetcomputer.org/)
- [Internet Computer forum](https://forum.dfinity.org/)
- [Vite developer docs](https://vitejs.dev/guide/)
- [React quick start guide](https://react.dev/learn)
- [`dfx.json` reference schema](https://internetcomputer.org/docs/current/references/dfx-json-reference/)
- [Rust developer docs](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [EVM RPC developer docs](https://internetcomputer.org/docs/current/developer-docs/integrations/ethereum/evm-rpc/)
- [Developer Experience Feedback Board](https://dx.internetcomputer.org/)

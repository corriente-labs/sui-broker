## How to launch the local Sui network, publish a Move package and test the proxy against it

### 1. Build the Sui binaries

* Get the Sui sources

  `git clone https://github.com/MystenLabs/sui`

* Switch to the revision of version 0.9.0

  `git checkout df05544fb0cbd6d6db016e71e5facb5e7cc27988`

* Build the binaries

  `cargo build --release`

* Copy binaries `sui` and `rpc-server` from `./target/release` to a directory mentioned in the PATH variable

### 2. Launch the local network and check addresses

* Remove old configs from directories

  `~/.sui` and `~/.move`, if any

* Create the initial config

  `sui genesis`

* Begin operations

  `sui start`

* Launch the RPC server

  `rpc-server`

* Show available addresses

  `sui client addresses`

* Show the active address

  `sui client active-address`

* Name one of the available addresses for later use

  `export SUI_ADDRESS=0x...`

* Show gas objects owned by the address

  `sui client gas --address $SUI_ADDRESS`

* Name one of the gas objects for later use

  `export SUI_GAS_ID=0x...`

### 3. Compile and publish the EVM

* Get the EVM sources

  `git clone https://github.com/corriente-labs/evm-sui-poc`

* Make sure the Move.toml file contains references to the same Sui revision

  `cat Move.toml | grep -i sui | grep rev`

* Compile and publish the EVM

  `sui client publish --path ./ --gas-budget 30000`
  
  => `Created Objects: - ID: 0x...`

* Name the EVM ID

  `export SUI_EVM_ID=0x...`

* Show the package

  `sui client object --id $SUI_EVM_ID`
  
  => `Modules: ["account", "state", "vm"]`

* Create the State object

  `sui client call --function create --module vm --package $SUI_EVM_ID --gas-budget 1000`
  
  => `Created Objects: - ID: 0x...`

* Name the State ID

  `export SUI_EVM_STATE_ID=0x...`

### 4. Call a EVM function from the proxy

* Get the proxy sources

  `git clone https://github.com/corriente-labs/hana-proxy`

* Launch the proxy

  `cargo run --release -- --socket-path ./p.sock --evm-driver-lib-path libsui_broker.so run-server`

* Issue command which eventually calls a function of the EVM

  `echo "hana-evm $SUI_EVM_ID $SUI_EVM_STATE_ID $SUI_GAS_ID $SUI_ADDRESS" | nc -U ./p.sock`

  The $SUI_ADDRESS serves as the signer of the transaction.

* If no errors, a SuiTransactionResponse object will be dumped to the proxy log

# Commands

Build the project using the following command:
```
cargo wasm
```
Run the unit test using the following command:
```
cargo unit-test
```

# Structure of the Project
```
.
└── .cargo
    └── config.toml
├── src
│   └── contract
│   │   └── exec.rs
│   │   └── helper.rs
│   │   └── query.rs
│   │   └── mod.rs
│   └── testing
│   │   └── tests.rs
│   │   └── mod.rs
│   └── error.rs
│   └── lib.rs
│   └── mock.rs
│   └── msg.rs
│   └── state.rs
├── README.MD
├── deploy_testnet.py
└── Cargo.toml

```
## .cargo/config.toml
cargo alias: `wasm` & `test`

## src
### /contract
This `contract.rs` file handles various functionalities related to NFT auctions in a smart contract. 

***/exec.rs***

1. `handle_cw721_auction_start`
- Handles the initialization of a CW721 token auction.
- Calls initialize_cw721_token_auction to set up the auction with the provided parameters.

2. `initialize_cw721_token_auction`
- Sets up the auction parameters such as start time, duration, minimum bid, etc.
- Checks for valid expiration times.
- Saves the auction details and state in the storage.

3. `submit_bid_for_auction`
- Allows users to submit bids for an ongoing auction.
- Validates the bid conditions such as auction state, funds provided, and bid amount.
- Updates the highest bid and bidder details.
- Refunds the previous highest bid if applicable.

4. `cancel_auction_and_refund`
- Allows the auction owner to cancel an ongoing auction.
- Transfers the NFT back to the owner.
- Refunds the highest bid if applicable.
- Marks the auction as cancelled in the storage.

5. `finalize_auction_and_transfer_assets`
- Finalizes the auction once it has ended.
- Transfers the NFT to the highest bidder.
- Transfers the bid amount to the auction owner.
- Handles cases where there are no bids or the auction is already claimed.

***/query.rs***

1. `get_auction_details`
- Fetches auction details based on optional filters such as token address and pagination parameters.

2. `get_bids_for_auction`
- Retrieves the list of bids for a specific auction.

3. `get_auction_state_by_id`
- Fetches the state of an auction by its ID.

***/helper.rs***
1. `fetch_and_update_next_auction_id`
- Fetches and increments the next auction ID in the storage.

2. `convert_milliseconds_to_expiration`
- Converts a timestamp in milliseconds to an Expiration type.

3. `set_expiration_from_block`
- Sets expiration time based on the current block information.

4. `fetch_latest_auction_state_for_token`
- Fetches the latest auction state for a specific token.

5. `query_token_owner`
- Queries the owner of a specific token using the CW721 contract.

### /lib.rs

***Entry Point***

1. `instantiate`
- Initializes the contract by setting the next auction ID to 1.
- Called when the contract is first deployed.

2. `execute`
- Handles different types of execution messages to perform actions like starting an auction, submitting a bid, canceling an auction, and finalizing an auction.

3. `query`
- Handles queries to fetch auction details, bids, and auction states.

### /msg.rs
1. `InstantiateMsg`
- Message used for instantiating the contract.
- Currently empty, indicating no specific initialization parameters are needed.

2. `ExecuteMsg`
- Enumeration of different execution messages that can be sent to the contract.
- Each variant represents a different action that can be performed.

3. `QueryMsg`
- Enumeration of different query messages that can be sent to the contract.
- Each variant represents a different type of query and specifies the expected return type.

4. `Cw721CustomMsg`
- Custom message type for initializing a CW721 token auction.

### /state.rs

***Structs***

1. `NFTAuctionState`
- Represents the state of an NFT auction.
- Contains information such as start and end times, highest bidder details, coin denomination, auction ID, minimum bid, owner, token ID, token address, and cancellation status.

2. `Bid`
- Holds auction details including auction IDs, token address, and token ID.
- Provides methods to access the latest auction ID and to add a new auction ID.

3. `AuctionDetails`
- Holds auction details including auction IDs, token address, and token ID.
- Provides methods to access the latest auction ID and to add a new auction ID.

4. `OrderBy`
- Enumeration to specify the order of results (Ascending or Descending).

***Storage***

1. `NEXT_AUCTION_ID`
- Stores the next available auction ID.

2. `BIDS`
- Maps auction IDs to a list of bids.

3. `NFT_AUCTION_STATE`
- Maps auction IDs to their corresponding NFTAuctionState.


### /mock.rs

- This mock.rs file is used for setting up a mock environment to facilitate testing of the CW721 NFT auction smart contract. It provides mock dependencies, custom query handlers, and handles specific token queries.

### /error.rs
- This error.rs file defines custom error types for the CW721 NFT auction smart contract. It uses the thiserror crate to create descriptive and structured error messages. These custom errors help in handling and debugging various issues that can arise during the execution of the contract.

### /testing/tests.rs
- Unit test for the contract

## deploy_testnet.py
- This Python script demonstrates how to deploy and instantiate a CW721 NFT marketplace contract on the Osmosis testnet using the cosmpy library.

# Deploy
## Prerequests
- Python3
- Osmosis testnet configuration
- Gas fee

## Command
```
python3 deploy_testnet.py
```
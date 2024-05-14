import os
from cosmpy.aio.client import LedgerClient
from cosmpy.crypto.address import Address
from cosmpy.crypto.keypairs import PrivateKey
from cosmpy.protos.cosmwasm.wasm.v1.tx_pb2 import MsgStoreCode, MsgInstantiateContract

# Configure network
client = LedgerClient('https://api.testnet.osmosis.zone')

# Load your wallet's private key
private_key = PrivateKey(os.environ['PRIVATE_KEY'])
my_address = Address(private_key.public_key())

# Load the compiled contract
with open("./target/wasm32-unknown-unknown/release/cw721_nft_marketplace.wasm", "rb") as file:
    wasm_byte_code = file.read()

# Create a MsgStoreCode transaction
store_code_msg = MsgStoreCode(
    sender=str(my_address),
    wasm_byte_code=wasm_byte_code
)

# Send transaction
tx_response = await client.tx_broadcast(store_code_msg, private_key)
code_id = tx_response.logs[0].events[0].attributes[0].value  # Get code ID from response

# Instantiate the contract
instantiate_msg = MsgInstantiateContract(
    sender=str(my_address),
    code_id=code_id,
    label='Example Contract Deployment',
    init_msg=b'{"count": 0}',  # Initialization parameters
)

# Send transaction to instantiate the contract
tx_instantiate_response = await client.tx_broadcast(instantiate_msg, private_key)
contract_address = tx_instantiate_response.logs[0].events[0].attributes[0].value

print(f'Contract deployed at {contract_address}')

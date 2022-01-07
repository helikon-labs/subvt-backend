# SubVT Validator List Server

Pub/sub WS RPC server for the validator list data. Serves the active validator list by default. Start with the 
`--inactive` flag to serve the inactive validator list.

Expects the `config` folder to be in the same folder as the executable.
Copy the `config` folder from inside the [subvt-config](../subvt-config) crate and edit the configuration.

Subscribe to the validator list using the `subscribe_validator_list` RPC method with no parameter.
`unsubscribe_validator_list` RPC method with the subscription id as the only parameter unsubscribes the client from
the update stream.

Sample subscription request:
```
{
    "id": 1,
    "jsonrpc": "2.0",
    "method": "subscribe_validator_list",
    "params": []
}
```

Sample subscription response with the subscription id in the `result` field:
```
{
    "jsonrpc": "2.0",
    "result": 6291495589394582,
    "id": 1
}
```

## Stream Specification

Stream data structure contains the list of new validators, updated validators and the ids of the ones to be removed:

```
{
    "finalized_block_number": 10505444,
    "insert": [
        ... list of new validators ...    
    ],
    "update":[
       ... list of updated validators ...
    ],
    "remove_ids":[
       ... list of the account ids of removed validators ...
    ]
}
```

After the initial subscription, the server is going to publish to the client a list of all the validators, i.e. the
`update` and `remove_ids`fields are going to be empty arrays in the initial response.

```
{
    "jsonrpc": "2.0",
    "method": "subscribe_validator_details",
    "params": {
        "subscription": 1478707943113707,
        "result": {
            "finalized_block_number": 10859839,
            "insert": [
                {
                    "account_id": "0x5C9EDF029BFAC2B480319B7F7D0558D1AFD0B690BB3FF3B95932D765E0899FED",
                    "controller_account_id": "0x8894084E5B48EDCC01FD9001C515EC301D80D0A6D773D9631BF0B768CE226766",
                    "confirmed": false,
                    "preferences": {
                        "commission_per_billion": 1000000000,
                        "blocks_nominations": false
                    },
                    "self_stake": {
                        "stash_account_id": "0x5C9EDF029BFAC2B480319B7F7D0558D1AFD0B690BB3FF3B95932D765E0899FED",
                        "active_amount": 100000000000
                    },
                    "is_active": true,
                    "active_next_session": true,
                    "inactive_nominations": {
                        "nomination_count": 1,
                        "total_amount": 80000000000000000
                    },
                    "oversubscribed": false,
                    "slash_count": 0,
                    "is_enrolled_in_1kv": false,
                    "is_parachain_validator": false,
                    "return_rate_per_billion": 0,
                    "blocks_authored": 0,
                    "heartbeat_received": false,
                    "validator_stake": {
                        "self_stake": 100000000000,
                        "total_stake": 5393248699168383,
                        "nominator_count": 1
                    }
                },
                ...
            ],
            "update": [],
            "remove_ids": []
        }
    }
}
```

After the initial data push, the client is going to receive only the changes, i.e. the new validators in the `insert`
field, the updated ones in the `update` field, and the ids of the ones to be removed in the `removed_ids` field.
# SubVT Network Status Server

Pub/sub WS RPC server for the live network status of the configured chain.

Expects the `config` folder to be in the same folder as the executable.
Copy the `config` folder from inside the [subvt-config](../subvt-config) crate and edit the configuration.

Subscribe to the network status with the  `subscribe_networkStatus` RPC method:

```
{
    "id": 1,
    "jsonrpc": "2.0",
    "method": "subscribe_networkStatus",
    "params": []
}
```

Sample subscription response with the subscription id in the `result` field:
```
{
    "jsonrpc": "2.0",
    "result": 2248230140339139,
    "id": 1
}
```

`unsubscribe_networkStatus` RPC method with the subscription id as the only parameter unsubscribes the client from
the update stream:

```
{
    "id":1,
    "jsonrpc":"2.0",
    "method": "unsubscribe_networkStatus",
    "params": [2248230140339139]
}
```

## Stream Specification

Initial response comes with the full network status:

```
{
    "jsonrpc": "2.0",
    "method": "subscribe_networkStatus",
    "params": {
        "subscription": 2248230140339139,
        "result": {
            "network": "kusama",
            "status": {
                "finalized_block_number": 10860783,
                "finalized_block_hash": "0xD49E7E2D71A503B4419D51825847C84E0795912331C9F35BE8ADF630B5C1D835",
                "best_block_number": 10860787,
                "best_block_hash": "0x05A7A145CDC9123CEA90ADB2BD8B749D068F07C0E23963C807EFF91B57030AEE",
                "active_era": {
                    "index": 3201,
                    "start_timestamp": 1641554874009,
                    "end_timestamp": 1641576474009
                },
                "current_epoch": {
                    "index": 18501,
                    "start_block_number": 10860297,
                    "start_timestamp": 1641565674,
                    "end_timestamp": 1641569274
                },
                "active_validator_count": 1000,
                "inactive_validator_count": 883,
                "last_era_total_reward": 594161089112028,
                "total_stake": 4956431691498674404,
                "return_rate_per_million": 175139,
                "min_stake": 3303010000000000,
                "max_stake": 21785704095217456,
                "average_stake": 4956431691498674,
                "median_stake": 4440718951873155,
                "era_reward_points": 1615200
            }
        }
    }
}
```

After the initial response, the server is going to publish to the client only the network status update block by
finalized block.

Status update is going to have only the updated data in the `diff` field. In the example below, only changes are
the best block number and hash, and the era reward points so far:

```
{
    "jsonrpc": "2.0",
    "method": "subscribe_networkStatus",
    "params": {
        "subscription": 2248230140339139,
        "result": {
            "network": "kusama",
            "diff": {
                "best_block_number": 10860795,
                "best_block_hash": "0x891228EB1D925878597B34723DC3CD9C036D373DB1DC63A1072E9BCC02C2DA34",
                "era_reward_points": 1620820
            }
        }
    }
}
```
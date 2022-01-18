# SubVT Validator Details Server

Pub/sub WS RPC server for validator detail data. Client subscribes for a single validator and receives data as
detailed below.

Expects the `config` folder to be in the same folder as the executable.
Copy the `config` folder from inside the [subvt-config](../subvt-config) crate and edit the configuration.

Subscribe to a validator's details using the `subscribe_validator_details` RPC method with the hex-encoded 32-byte
validator account id as the only parameter:

```
{
    "id": 1,
    "jsonrpc": "2.0",
    "method": "subscribe_validatorDetails",
    "params": [
        "0x00BA7F0D55312D16758EAC1F9D9285FD62CDEFED9FEE1C0312A87A401AFCEF25"
    ]
}
```

Sample subscription response with the subscription id in the `result` field:
```
{
    "jsonrpc": "2.0",
    "result": 1672621751564676,
    "id": 1
}
```

`unsubscribe_validatorDetails` RPC method with the subscription id as the only parameter unsubscribes the client from
the update stream:

```
{
    "id": 1,
    "jsonrpc": "2.0",
    "method": "unsubscribe_validatorDetails",
    "params": [1672621751564676]
}
```

## Stream Specification

Initial response comes with the full validator details:

```
{
    "jsonrpc": "2.0",
    "method": "subscribe_validatorDetails",
    "params": {
        "subscription": 1478707943113707,
        "result": {
            "finalized_block_number": 10860357,
            "validator_details": {
                "account": {
                    "id": "0x00BA7F0D55312D16758EAC1F9D9285FD62CDEFED9FEE1C0312A87A401AFCEF25"
                },
                "controller_account_id": "0x08820C4A9C688A385576C8851F5D8CDF3D143F5C79A79D9BBB5BC28A3DE02C08",
                "preferences": {
                    "commission_per_billion": 1000000000,
                    "blocks_nominations": false
                },
                "self_stake": {
                    "stash_account_id": "0x00BA7F0D55312D16758EAC1F9D9285FD62CDEFED9FEE1C0312A87A401AFCEF25",
                    "total_amount": 10000000000,
                    "active_amount": 10000000000
                },
                "reward_destination": {
                    "Account": "0xF61FC9541E02E4FA9E6EEB3CC077938352968277C986EEEAD12149EEF1A9F477"
                },
                "next_session_keys": "0xED3138E38B1454AB1AB9008F293627EDCF445D87A3E1E2258565ECC615014941F44DF689A3D5ACE5B6A109E35AC63D1E62B52E3A1A582CE0FA0E220FC8B6715798154246F5EAE6326652CA05D53ED085DC98ABA3DBD2F8013A6F5FF7A77E885036C1733C66FFE9C61E4789438C4A75DE89C92E25A7C7B90E4C919F9E0960EF6EEE7A115F9E7FFBB13F392377F2BBA6D78115DB1D8A2D68F647BA8EA712CD5F657A1B9A18A92F9E61434977BDC84938E9CB9361790ED9947FED2E31CE9FD1D850",
                "is_active": true,
                "active_next_session": true,
                "nominations": [{
                    "stash_account_id": "0x16FB068D0CB5B89CE88CC2A086E3B336439F33844391E76ABB1EEB2F83198C50",
                    "submission_era_index": 2359,
                    "target_account_ids": ["0x02948B18CD5001E68A33499343BD8ED974FC8398BBFDC3DFAFBC7C478544F67D", "0x027D4770A8FB3EE70BB1D12B64F93C794DAE984ED5E991267C7E776C77470F5A", "0x0217602B2E783DAB8C95B8F0AADCC0399DEF3F850DE2AC25FCF008ACBDF53A59", "0x0210F894089A5FD3E91BCC8F6848717C556F002438CD902C40AF4D031E9D5E21", "0x02098B5F718885F0D6F0F18359A7D16B44C9229857934EFE66DAF4D9F0EB7A43", "0x01E8D0AF5354D51139D75961106D760413BB912CD7A8BC96FBB5319E43FD0077", "0x00BBC6562DF1CFD3219CFF334DF3AD5AEF49B593CEF95D14AEBA63B8E8D9795A", "0x00BA7F0D55312D16758EAC1F9D9285FD62CDEFED9FEE1C0312A87A401AFCEF25", "0x0071D7EA9D85CF0A22365CD41CCA3F63121FBEF0E1EEA72109DB9B12AF8D4860", "0x007B26343E6EBAED9459FFAAE9358C5B2460902C2A0D63D68A748E1D8EB15033", "0x001757A13F2774A388540AA0CD306DC4B7B47CEFDF0B5388C9FA6336DF79FE99", "0x0000966D74F8027E07B43717B6876D97544FE0D71FACEF06ACC8382749AE944E", "0x02A6ED2142A394D0F42A51281478530276F88D15657AA277B62C54BD1576F322", "0x02C1151878EA5C35D75C7D4F879FB48EA7A4199E2D3AC9ED79D686A157384D2F", "0x041EDA8CF4EA9ED59862C6F5AD3A51E7EF62EFF19B629CB241A10D5FD7F96D22", "0x04387557367A752C19AD0636FFA9F4D9E2DCA0B112A808EB53D2615CB9992C3D"],
                    "stake": {
                        "stash_account_id": "0x16FB068D0CB5B89CE88CC2A086E3B336439F33844391E76ABB1EEB2F83198C50",
                        "total_amount": 80000000000,
                        "active_amount": 80000000000
                    }
                }, {
                    "stash_account_id": "0xCC68205D321E71A9C9DA3BBC6E4A336DF6C1469567C42EED7382B6D06AB0D011",
                    "submission_era_index": 2358,
                    "target_account_ids": ["0x0000966D74F8027E07B43717B6876D97544FE0D71FACEF06ACC8382749AE944E", "0x001757A13F2774A388540AA0CD306DC4B7B47CEFDF0B5388C9FA6336DF79FE99", "0x0071D7EA9D85CF0A22365CD41CCA3F63121FBEF0E1EEA72109DB9B12AF8D4860", "0x007B26343E6EBAED9459FFAAE9358C5B2460902C2A0D63D68A748E1D8EB15033", "0x00BA7F0D55312D16758EAC1F9D9285FD62CDEFED9FEE1C0312A87A401AFCEF25", "0x02098B5F718885F0D6F0F18359A7D16B44C9229857934EFE66DAF4D9F0EB7A43", "0x0210F894089A5FD3E91BCC8F6848717C556F002438CD902C40AF4D031E9D5E21", "0x02948B18CD5001E68A33499343BD8ED974FC8398BBFDC3DFAFBC7C478544F67D", "0x02C1151878EA5C35D75C7D4F879FB48EA7A4199E2D3AC9ED79D686A157384D2F", "0x048BC4DA5E4289EEB52244056C70C535F165D37AD921C08B9D00466A1FBFE442", "0x0600CF06F1C8E912C9D968F1DE9FE6ED32D033B8E87BFAC698602C25604E2B4E", "0x086F2422947FDBEBD39A68F8708064BD5D9CAAB70D1D6A51ABFF895DB91F5655", "0x0A50BD5098BFC45338FED742FC7A935AFF9ED058C5377B20116585907F106201", "0x0A88006A747B712BBAC65DC015105C9CB6D29BA60DDE6762A881975C6B1B1A02", "0x0AD8B4F94DC8E6229E2C448D995094CDC1BD356DC14C6155467AC07EDE18B872", "0x102E184CAB1726546391446BFB60B78B2FA78A927F7BF2BD4A734CFEB62C8909"],
                    "stake": {
                        "stash_account_id": "0xCC68205D321E71A9C9DA3BBC6E4A336DF6C1469567C42EED7382B6D06AB0D011",
                        "total_amount": 30000000000,
                        "active_amount": 30000000000
                    }
                },
                ... rest of the nominations ...
                ],
                "oversubscribed": false,
                "active_era_count": 0,
                "inactive_era_count": 0,
                "slash_count": 0,
                "offline_offence_count": 0,
                "total_reward_points": 0,
                "unclaimed_era_indices": [],
                "is_parachain_validator": false,
                "return_rate_per_billion": 0,
                "blocks_authored": 0,
                "heartbeat_received": false,
                "validator_stake": {
                    "account": {
                        "id": "0x00BA7F0D55312D16758EAC1F9D9285FD62CDEFED9FEE1C0312A87A401AFCEF25"
                    },
                    "self_stake": 10000000000,
                    "total_stake": 3510664175579065,
                    "nominators": [{
                        "account": {
                            "id": "0x48B7BE5B47635148BA704E935BF1AE99645A925BA0207CB856EB8061B598194A"
                        },
                        "stake": 1999934333929
                    }, {
                        "account": {
                            "id": "0xBCA57021271ED0D131E3AD723E1176A9EACF1469329CBC690ED8C75E0B2D9207"
                        },
                        "stake": 20000000000
                    }, {
                        "account": {
                            "id": "0x02EC920BBEB83F513D1E7064FB46C094621D4B1D622071895DBE24D7D36F8C62"
                        },
                        "stake": 3508634241245136
                    }]
                }
            }
        }
    }
}
```

After the initial response, the server is going to publish to the client only the updates regarding the validator.
Validator update is going to have only the changed fields. In the case of a validator with no changes, the response will
only contain the last processed finalized block number:

```
{
    "jsonrpc": "2.0",
    "method": "subscribe_validatorDetails",
    "params": {
        "subscription": 1478707943113707,
        "result": {
            "finalized_block_number": 10860365
        }
    }
}
```
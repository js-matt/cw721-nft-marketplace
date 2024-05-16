#[cfg(test)]
mod tests {
    use crate::{
        error::ContractError,
        execute, instantiate,
        mock::{custom_mock_dependencies, MOCK_TOKEN_ADDR, MOCK_TOKEN_OWNER, MOCK_UNCLAIMED_TOKEN},
        msg::Cw721CustomMsg,
        query,
        state::{
            get_bids, load_auction_details, load_bids, load_next_auction_id,
            load_nft_auction_state, read_auction_details, save_auction_details, save_bids,
            save_next_auction_id, save_nft_auction_state, AuctionDetails, Bid, NFTAuctionState,
            OrderBy,
        },
        ExecuteMsg, InstantiateMsg, QueryMsg,
    };
    use cosmwasm_std::{
        attr, coins, from_json,
        testing::{mock_dependencies, mock_env, mock_info},
        to_json_binary, Addr, CosmosMsg, Deps, DepsMut, Response, Timestamp, Uint128, WasmMsg,
    };

    use cw721::{Cw721ExecuteMsg, Cw721ReceiveMsg};

    #[test]
    fn test_save_and_load_bids() {
        let mut deps = mock_dependencies();
        let auction_id = 1u128;
        let bids = vec![
            Bid {
                bidder: "Alice".to_string(),
                amount: Uint128::new(100),
                timestamp: Timestamp::from_seconds(100),
            },
            Bid {
                bidder: "Bob".to_string(),
                amount: Uint128::new(150),
                timestamp: Timestamp::from_seconds(200),
            },
        ];

        save_bids(&mut deps.storage, auction_id, bids.clone()).unwrap();
        let loaded_bids = load_bids(&deps.storage, auction_id).unwrap();
        assert_eq!(loaded_bids, bids);
    }

    #[test]
    fn test_save_and_load_nft_auction_state() {
        let mut deps = mock_dependencies();
        let auction_id = 1u128;
        let auction_state = NFTAuctionState {
            start: Timestamp::from_seconds(100),
            end: Timestamp::from_seconds(200),
            high_bidder_addr: Addr::unchecked("high_bidder"),
            high_bidder_amount: Uint128::new(100),
            coin_denomination: "uusd".to_string(),
            auction_id: Uint128::new(1),
            min_bid: Some(Uint128::new(50)),
            owner: "owner".to_string(),
            token_id: "token1".to_string(),
            token_address: "token_address".to_string(),
            is_cancelled: false,
        };

        save_nft_auction_state(&mut deps.storage, auction_id, auction_state.clone()).unwrap();
        let loaded_auction_state = load_nft_auction_state(&deps.storage, auction_id).unwrap();
        assert_eq!(loaded_auction_state, auction_state);
    }

    #[test]
    fn test_save_and_load_next_auction_id() {
        let mut deps = mock_dependencies();
        let next_auction_id = Uint128::new(2);

        save_next_auction_id(&mut deps.storage, next_auction_id).unwrap();
        let loaded_next_auction_id = load_next_auction_id(&deps.storage).unwrap();
        assert_eq!(loaded_next_auction_id, next_auction_id);
    }

    #[test]
    fn test_save_and_load_auction_details() {
        let mut deps = mock_dependencies();
        let pk = "auction1".to_string();
        let details = AuctionDetails {
            auction_ids: vec![Uint128::new(1), Uint128::new(2)],
            token_address: "token_address".to_string(),
            token_id: "token1".to_string(),
        };

        save_auction_details(&mut deps.storage, pk.clone(), details.clone()).unwrap();
        let loaded_details = load_auction_details(&mut deps.storage, &pk).unwrap();
        assert_eq!(loaded_details, details);
    }

    fn check_auction_created(deps: Deps, min_bid: Option<Uint128>) {
        assert_eq!(
            NFTAuctionState {
                start: Timestamp::from_seconds(100000),
                end: Timestamp::from_seconds(200000),
                high_bidder_addr: Addr::unchecked(""),
                high_bidder_amount: Uint128::zero(),
                coin_denomination: "usd".to_string(),
                auction_id: 1u128.into(),
                owner: MOCK_TOKEN_OWNER.to_string(),
                token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
                token_address: MOCK_TOKEN_ADDR.to_owned(),
                is_cancelled: false,
                min_bid,
            },
            load_nft_auction_state(deps.storage, 1u128).unwrap()
        );
    }

    fn start_auction(deps: DepsMut, min_bid: Option<Uint128>) {
        let custom_msg = Cw721CustomMsg::InitializeCW721TokenAuction {
            start_time: 100000,
            duration: 100000,
            coin_denomination: "usd".to_string(),
            min_bid,
        };
        let msg = ExecuteMsg::AuctionStart(Cw721ReceiveMsg {
            sender: MOCK_TOKEN_OWNER.to_owned(),
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            msg: to_json_binary(&custom_msg).unwrap(),
        });
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0u64);

        let info = mock_info(MOCK_TOKEN_ADDR, &[]);
        let _res = execute(deps, env, info, msg).unwrap();
    }

    #[test]
    fn test_initialize_cw721_token_auction() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("owner", &[]);
        let _res = instantiate(deps.as_mut(), env, info, InstantiateMsg {}).unwrap();

        let custom_msg = Cw721CustomMsg::InitializeCW721TokenAuction {
            start_time: 100000,
            duration: 100000,
            coin_denomination: "usd".to_string(),
            min_bid: None,
        };
        let msg = ExecuteMsg::AuctionStart(Cw721ReceiveMsg {
            sender: MOCK_TOKEN_OWNER.to_owned(),
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            msg: to_json_binary(&custom_msg).unwrap(),
        });
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0u64);

        let info = mock_info(MOCK_TOKEN_ADDR, &[]);
        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(
            res,
            Response::new().add_attributes(vec![
                attr("action", "start_auction"),
                attr("start_time", "100000.000000000"),
                attr("end_time", "200000.000000000"),
                attr("coin_denomination", "usd"),
                attr("auction_id", "1"),
            ]),
        );
        check_auction_created(deps.as_ref(), None);
    }

    #[test]
    fn test_submit_bid_for_auction_non_existing_auction() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info(MOCK_TOKEN_OWNER, &[]);
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::SubmitBid {
            token_id: MOCK_UNCLAIMED_TOKEN.to_string(),
            token_address: MOCK_TOKEN_ADDR.to_string(),
        };
        let info = mock_info("bidder", &coins(100, "usd"));
        let res = execute(deps.as_mut(), env, info, msg);
        assert_eq!(ContractError::AuctionDoesNotExist {}, res.unwrap_err());
    }

    #[test]
    fn test_submit_bid_for_auction_ended_auction() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info(MOCK_TOKEN_OWNER, &[]);
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        start_auction(deps.as_mut(), None);
        check_auction_created(deps.as_ref(), None);

        let msg = ExecuteMsg::SubmitBid {
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            token_address: MOCK_TOKEN_ADDR.to_string(),
        };

        env.block.time = Timestamp::from_seconds(300);

        let info = mock_info("sender", &coins(100, "usd".to_string()));
        let res = execute(deps.as_mut(), env, info, msg);
        assert_eq!(ContractError::AuctionEnded {}, res.unwrap_err());
    }

    #[test]
    fn test_initialize_cw721_token_auction_zero_start_time() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        let hook_msg = Cw721CustomMsg::InitializeCW721TokenAuction {
            start_time: 0,
            duration: 1,
            coin_denomination: "usd".to_string(),
            min_bid: None,
        };
        let msg = ExecuteMsg::AuctionStart(Cw721ReceiveMsg {
            sender: MOCK_TOKEN_OWNER.to_owned(),
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            msg: to_json_binary(&hook_msg).unwrap(),
        });
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);

        let info = mock_info(MOCK_TOKEN_ADDR, &[]);
        let res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(ContractError::InValidTime {}, res.unwrap_err());
    }

    #[test]
    fn test_initialize_cw721_token_auction_zero_duration() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        let hook_msg = Cw721CustomMsg::InitializeCW721TokenAuction {
            start_time: 100,
            duration: 0,
            coin_denomination: "usd".to_string(),
            min_bid: None,
        };
        let msg = ExecuteMsg::AuctionStart(Cw721ReceiveMsg {
            sender: MOCK_TOKEN_OWNER.to_owned(),
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            msg: to_json_binary(&hook_msg).unwrap(),
        });
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0);

        let info = mock_info(MOCK_TOKEN_ADDR, &[]);
        let res = execute(deps.as_mut(), env, info, msg);

        assert_eq!(ContractError::InValidTime {}, res.unwrap_err());
    }

    #[test]
    fn test_cancel_auction_and_refund_no_bids() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        start_auction(deps.as_mut(), None);

        let msg = ExecuteMsg::CancelAuctionAndRefund {
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            token_address: MOCK_TOKEN_ADDR.to_string(),
        };

        env.block.time = Timestamp::from_seconds(200000);

        let info = mock_info(MOCK_TOKEN_OWNER, &[]);
        let res = execute(deps.as_mut(), env, info, msg).unwrap();

        assert_eq!(
            Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: MOCK_TOKEN_ADDR.to_owned(),
                msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
                    recipient: MOCK_TOKEN_OWNER.to_owned(),
                    token_id: MOCK_UNCLAIMED_TOKEN.to_owned()
                })
                .unwrap(),
                funds: vec![],
            })),
            res
        );

        assert!(
            load_nft_auction_state(deps.as_ref().storage, 1u128)
                .unwrap()
                .is_cancelled
        );
    }

    #[test]
    fn test_cancel_auction_and_refund_not_token_owner() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        start_auction(deps.as_mut(), None);

        let msg = ExecuteMsg::CancelAuctionAndRefund {
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            token_address: MOCK_TOKEN_ADDR.to_string(),
        };

        env.block.time = Timestamp::from_seconds(150);

        let info = mock_info("anyone", &[]);
        let res = execute(deps.as_mut(), env, info, msg);
        assert_eq!(ContractError::Unauthorized {}, res.unwrap_err());
    }

    #[test]
    fn test_cancel_auction_and_refund_ended_auction() {
        let mut deps = mock_dependencies();
        let mut env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        start_auction(deps.as_mut(), None);

        let msg = ExecuteMsg::CancelAuctionAndRefund {
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            token_address: MOCK_TOKEN_ADDR.to_string(),
        };

        env.block.time = Timestamp::from_seconds(300);

        let info = mock_info(MOCK_TOKEN_OWNER, &[]);
        let res = execute(deps.as_mut(), env, info, msg);
        assert_eq!(ContractError::AuctionEnded {}, res.unwrap_err());
    }

    #[test]
    fn test_finalize_auction_and_transfer_assets_no_bids() {
        let mut deps = custom_mock_dependencies(&[]);
        let mut env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();

        start_auction(deps.as_mut(), None);

        env.block.time = Timestamp::from_seconds(250);

        let msg = ExecuteMsg::FinalizeAuctionAndTranferAssets {
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            token_address: MOCK_TOKEN_ADDR.to_string(),
        };

        let info = mock_info("any_user", &[]);
        let res = execute(deps.as_mut(), env, info, msg).unwrap();
        assert_eq!(
            Response::new()
                .add_message(CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: MOCK_TOKEN_ADDR.to_owned(),
                    msg: to_json_binary(&Cw721ExecuteMsg::TransferNft {
                        recipient: MOCK_TOKEN_OWNER.to_owned(),
                        token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
                    })
                    .unwrap(),
                    funds: vec![],
                }))
                .add_attribute("action", "claim")
                .add_attribute("token_id", MOCK_UNCLAIMED_TOKEN)
                .add_attribute("token_contract", MOCK_TOKEN_ADDR)
                .add_attribute("recipient", MOCK_TOKEN_OWNER)
                .add_attribute("winning_bid_amount", Uint128::zero())
                .add_attribute("auction_id", "1"),
            res
        );
    }

    #[test]
    fn test_finalize_auction_and_transfer_assets_auction_already_claimed() {
        let mut deps = custom_mock_dependencies(&[]);
        let env = mock_env();
        let info = mock_info("owner", &[]);
        let msg = InstantiateMsg {};
        let _res = instantiate(deps.as_mut(), env, info, msg).unwrap();

        let hook_msg = Cw721CustomMsg::InitializeCW721TokenAuction {
            start_time: 100000,
            duration: 100000,
            coin_denomination: "usd".to_string(),
            min_bid: None,
        };
        let msg = ExecuteMsg::AuctionStart(Cw721ReceiveMsg {
            sender: MOCK_TOKEN_OWNER.to_owned(),
            token_id: "claimed_token".to_string(),
            msg: to_json_binary(&hook_msg).unwrap(),
        });
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0u64);

        let info = mock_info(MOCK_TOKEN_ADDR, &[]);
        let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        // Auction is over.
        env.block.time = Timestamp::from_seconds(300);

        let msg = ExecuteMsg::FinalizeAuctionAndTranferAssets {
            token_id: "claimed_token".to_string(),
            token_address: MOCK_TOKEN_ADDR.to_string(),
        };

        let info = mock_info("any_user", &[]);
        let res = execute(deps.as_mut(), env, info, msg);
        assert_eq!(ContractError::AuctionAlreadyClaimed {}, res.unwrap_err());
    }

    #[test]
    fn test_query_start_auction() {
        let mut deps = mock_dependencies();
        let env = mock_env();
        let info = mock_info("owner", &[]);
        let _res = instantiate(deps.as_mut(), env, info, InstantiateMsg {}).unwrap();

        let custom_msg = Cw721CustomMsg::InitializeCW721TokenAuction {
            start_time: 100000,
            duration: 100000,
            coin_denomination: "usd".to_string(),
            min_bid: None,
        };
        let msg = ExecuteMsg::AuctionStart(Cw721ReceiveMsg {
            sender: MOCK_TOKEN_OWNER.to_owned(),
            token_id: MOCK_UNCLAIMED_TOKEN.to_owned(),
            msg: to_json_binary(&custom_msg).unwrap(),
        });
        let mut env = mock_env();
        env.block.time = Timestamp::from_seconds(0u64);

        let info = mock_info(MOCK_TOKEN_ADDR, &[]);
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        let msg = ExecuteMsg::AuctionStart(Cw721ReceiveMsg {
            sender: "foo_token_owner".to_owned(),
            token_id: "foo_token".to_owned(),
            msg: to_json_binary(&custom_msg).unwrap(),
        });

        let info = mock_info(MOCK_TOKEN_ADDR, &[]);
        execute(deps.as_mut(), env.clone(), info, msg).unwrap();

        check_auction_created(deps.as_ref(), None);

        let query_msg = QueryMsg::AuctionDetails {
            token_address: Some(MOCK_TOKEN_ADDR.to_string()),
            start_after: Some("e".to_string()),
            limit: Some(10),
        };
        let res: Vec<AuctionDetails> =
            from_json(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        assert_eq!(
            vec![
                AuctionDetails {
                    auction_ids: vec![Uint128::from(2u128)],
                    token_address: MOCK_TOKEN_ADDR.to_string(),
                    token_id: "foo_token".to_string(),
                },
                AuctionDetails {
                    auction_ids: vec![Uint128::from(1u128)],
                    token_address: MOCK_TOKEN_ADDR.to_string(),
                    token_id: "mock_unclaimed_token".to_string(),
                }
            ],
            res
        );

        let query_msg = QueryMsg::AuctionDetails {
            token_address: Some(MOCK_TOKEN_ADDR.to_string()),
            start_after: Some("g".to_string()),
            limit: Some(10),
        };
        let res: Vec<AuctionDetails> =
            from_json(&query(deps.as_ref(), env.clone(), query_msg).unwrap()).unwrap();
        assert_eq!(
            vec![AuctionDetails {
                auction_ids: vec![Uint128::from(1u128)],
                token_address: MOCK_TOKEN_ADDR.to_string(),
                token_id: "mock_unclaimed_token".to_string(),
            }],
            res
        );

        let query_msg = QueryMsg::AuctionDetails {
            token_address: None,
            start_after: None,
            limit: Some(10),
        };
        let res: Vec<AuctionDetails> =
            from_json(&query(deps.as_ref(), env, query_msg).unwrap()).unwrap();
        assert_eq!(
            vec![
                AuctionDetails {
                    auction_ids: vec![Uint128::from(2u128)],
                    token_address: MOCK_TOKEN_ADDR.to_string(),
                    token_id: "foo_token".to_string(),
                },
                AuctionDetails {
                    auction_ids: vec![Uint128::from(1u128)],
                    token_address: MOCK_TOKEN_ADDR.to_string(),
                    token_id: "mock_unclaimed_token".to_string(),
                }
            ],
            res
        );
    }

    #[test]
    fn test_get_bids_defaults() {
        let mut deps = mock_dependencies();
        let auction_id: u128 = 1;
        // Assume some Bids are already stored
        let sample_bids = vec![
            Bid {
                bidder: "Alice".to_string(),
                amount: Uint128::from(100u128),
                timestamp: Timestamp::from_seconds(1000),
            },
            Bid {
                bidder: "Bob".to_string(),
                amount: Uint128::from(200u128),
                timestamp: Timestamp::from_seconds(1001),
            },
        ];
        save_bids(&mut deps.storage, auction_id, sample_bids).unwrap();

        let result = get_bids(&deps.storage, auction_id, None, None, None).unwrap();
        assert_eq!(result.len(), 2); // Default limit is 20, so should show both bids
        assert_eq!(result[0].bidder, "Alice");
    }

    #[test]
    fn test_get_bids_desc_order() {
        let mut deps = mock_dependencies();
        let auction_id: u128 = 1;
        let sample_bids = vec![
            Bid {
                bidder: "Alice".to_string(),
                amount: Uint128::from(100u128),
                timestamp: Timestamp::from_seconds(1000),
            },
            Bid {
                bidder: "Bob".to_string(),
                amount: Uint128::from(200u128),
                timestamp: Timestamp::from_seconds(1001),
            },
        ];
        save_bids(&mut deps.storage, auction_id, sample_bids).unwrap();

        let result = get_bids(
            &deps.storage,
            auction_id,
            None,
            Some(10),
            Some(OrderBy::Desc),
        )
        .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].bidder, "Bob"); // Order should be descending by timestamp
    }

    #[test]
    fn test_read_auction_details_no_token_address() {
        let mut deps = mock_dependencies();

        // Setup an example AuctionDetails entry
        let details = AuctionDetails {
            auction_ids: vec![Uint128::new(1)],
            token_address: "cosmos_token_address".to_string(),
            token_id: "cosmos_token_id".to_string(),
        };
        save_auction_details(
            &mut deps.storage,
            "cosmos_token_address".to_string(),
            details,
        )
        .unwrap();

        let result = read_auction_details(&deps.storage, None, None, None).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].token_address, "cosmos_token_address");
    }

    #[test]
    fn test_read_auction_details_with_token_address() {
        let mut deps = mock_dependencies();

        // Setup multiple AuctionDetails entries
        let details = AuctionDetails {
            auction_ids: vec![Uint128::new(1), Uint128::new(2)],
            token_address: "specific_token_address".to_string(),
            token_id: "specific_token_id".to_string(),
        };
        save_auction_details(
            &mut deps.storage,
            "specific_token_address".to_string(),
            details,
        )
        .unwrap();

        let result = read_auction_details(
            &deps.storage,
            Some("specific_token_address".to_string()),
            None,
            None,
        )
        .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].token_id, "specific_token_id");
    }
}

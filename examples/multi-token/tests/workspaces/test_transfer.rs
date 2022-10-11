#[cfg(test)]
mod tests {
    use near_sdk::json_types::U128;
    use near_contract_standards::multi_token::token::Token;
    use near_sdk::ONE_YOCTO;
    use near_units::parse_near;
    use workspaces::AccountId;
    use near_contract_standards::storage_management::StorageBalanceBounds;

    use crate::utils::{helper_mint, init, register_user_for_token};

    #[tokio::test]
    async fn simulate_mt_transfer_with_approval() -> anyhow::Result<()> {
        let worker = workspaces::sandbox().await?;
        let (mt, alice, bob, _) = init(&worker).await?;

        let charlie = mt
            .as_account()
            .create_subaccount("charlie")
            .initial_balance(parse_near!("10 N"))
            .transact()
            .await?
            .into_result()?;

        let token: Token = helper_mint(&mt, alice.id().clone(), 1000u128, "title1".to_string(), "desc1".to_string()).await?;

        // Grant bob an approval to take 50 of alice's tokens.
        let _ = alice.call(mt.id(), "mt_approve")
            .args_json((
                [token.token_id.clone()],
                [U128(50)],
                bob.id(),
                Option::<String>::None,
            ))
            .gas(300_000_000_000_000)
            .deposit(490000000000000000000)
            .transact()
            .await?;


        let balance_bounds = mt.view("storage_balance_bounds", vec![])
            .await?
            .json::<StorageBalanceBounds>()?;

        register_user_for_token(&mt, charlie.id(), balance_bounds.min.into()).await?;

        // Bob tries to transfer 50 tokens to charlie
        let res = bob
            .call(mt.id(), "mt_transfer")
            .args_json((
                charlie.id(),
                token.token_id.clone(),
                "50",
                Option::<(AccountId, u64)>::Some((alice.id().clone(), 0)),
                Option::<String>::None,
            ))
            .gas(300_000_000_000_000)
            .deposit(ONE_YOCTO)
            .transact()
            .await?;

        assert!(res.is_success());

        // Bob tries to transfer 50 tokens to charlie, but fails because of insufficient approval.
        let res = bob
            .call(mt.id(), "mt_transfer")
            .args_json((
                charlie.id(),
                token.token_id.clone(),
                "50",
                Option::<(AccountId, u64)>::Some((alice.id().clone(), 0)),
                Option::<String>::None,
            ))
            .gas(300_000_000_000_000)
            .deposit(ONE_YOCTO)
            .transact()
            .await?;

        assert!(res.is_failure());

        Ok(())
    }


    #[tokio::test]
    async fn simulate_mt_transfer_wrong_approval() -> anyhow::Result<()> {
        let worker = workspaces::sandbox().await?;
        let (mt, alice, bob, _) = init(&worker).await?;

        let charlie = mt
            .as_account()
            .create_subaccount("charlie")
            .initial_balance(parse_near!("10 N"))
            .transact()
            .await?
            .into_result()?;

        let token: Token = helper_mint(&mt, alice.id().clone(), 1000u128, "title1".to_string(), "desc1".to_string()).await?;

        // Grant bob an approval to take 50 of alice's tokens.
        let _ = alice.call(mt.id(), "mt_approve")
            .args_json((
                [token.token_id.clone()],
                [U128(50)],
                bob.id(),
                Option::<String>::None,
            ))
            .gas(300_000_000_000_000)
            .deposit(490000000000000000000)
            .transact()
            .await?;

        // register charlie
        let _ = charlie.call(mt.id(), "register")
            .args_json((
                token.token_id.clone(),
                charlie.id(),
            ))
            .gas(300_000_000_000_000)
            .transact()
            .await?;

        // charlie tries to transfer 50 tokens to himself, but fails because of wrong approval.
        let res = charlie
            .call(mt.id(), "mt_transfer")
            .args_json((
                charlie.id(),
                token.token_id.clone(),
                "50",
                Option::<(AccountId, u64)>::Some((alice.id().clone(), 0)),
                Option::<String>::None,
            ))
            .gas(300_000_000_000_000)
            .deposit(ONE_YOCTO)
            .transact()
            .await?;

        println!("res = {:?}", res);
        assert!(res.is_failure());

        Ok(())
    }
}

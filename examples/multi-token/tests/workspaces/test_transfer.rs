use near_contract_standards::multi_token::token::Token;
use near_sdk::ONE_YOCTO;
use near_units::parse_near;
use workspaces::AccountId;

use crate::utils::{helper_mint, init};

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
    let res = alice.call(mt.id(), "mt_approve")
        .args_json((
            [token.token_id.clone()],
            [50u64],
            bob.id(),
            Option::<String>::None,
        ))
        .gas(300_000_000_000_000)
        .deposit(490000000000000000000)
        .transact()
        .await?;
    println!("res: {:?}", res);

    // register charlie
    let _ = charlie.call(mt.id(), "register")
        .args_json((
            token.token_id.clone(),
            charlie.id(),
        ))
        .gas(300_000_000_000_000)
        .transact()
        .await?;

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

    assert!(res.is_success()); // todo: this should fail

    Ok(())
}

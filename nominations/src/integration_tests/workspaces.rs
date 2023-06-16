use near_units::parse_near;
use serde_json::json;
use workspaces::{Account, Contract, DevNetwork, Worker};

use crate::{storage::HouseType, TokenMetadata, SECOND};

async fn init(
    worker: &Worker<impl DevNetwork>,
) -> anyhow::Result<(Contract, Account, Account, Account, Account)> {
    // deploy contracts
    let ndc_nominations_contract = worker
        .dev_deploy(include_bytes!("../../../res/ndc_nominations.wasm"))
        .await?;

    let registry_contract = worker
        .dev_deploy(include_bytes!("../../../res/registry.wasm"))
        .await?;

    let authority_acc = worker.dev_create_account().await?;
    let iah_issuer = worker.dev_create_account().await?;
    let alice_acc = worker.dev_create_account().await?;
    let bob_acc = worker.dev_create_account().await?;
    let john_acc = worker.dev_create_account().await?;
    let elon_acc = worker.dev_create_account().await?;

    // get current block time
    let block_info = worker.view_block().await?;
    let current_timestamp = block_info.timestamp();
    let end_time = current_timestamp + 60 * SECOND;

    // initialize contracts
    let res  = ndc_nominations_contract
        .call("new")
        .args_json(json!({"sbt_registry": registry_contract.id(),"iah_issuer": iah_issuer.id(),"og_class": (iah_issuer.id(),2), "admins": [authority_acc.id()], "start_time": 0, "end_time": end_time,}))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let res = registry_contract
        .call("new")
        .args_json(json!({"authority": authority_acc.id(),"iah_issuer": iah_issuer.id(), "iah_classes": [1],}))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // add iah_issuer
    let res = authority_acc
        .call(registry_contract.id(), "admin_add_sbt_issuer")
        .args_json(json!({"issuer": iah_issuer.id()}))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // mint IAH and OG sbt to alice
    let alice_tokens = vec![
        TokenMetadata {
            class: 1,
            issued_at: Some(0),
            expires_at: None,
            reference: None,
            reference_hash: None,
        },
        TokenMetadata {
            class: 2,
            issued_at: Some(0),
            expires_at: None,
            reference: None,
            reference_hash: None,
        },
    ];

    // mint only IAH to bob
    let bob_tokens = vec![TokenMetadata {
        class: 1,
        issued_at: Some(0),
        expires_at: Some(current_timestamp),
        reference: None,
        reference_hash: None,
    }];

    // mint only OG to john
    let john_tokens = vec![TokenMetadata {
        class: 2,
        issued_at: Some(0),
        expires_at: Some(current_timestamp),
        reference: None,
        reference_hash: None,
    }];

    // mint expired OG and expired IAH to elon
    let elon_tokens = vec![
        TokenMetadata {
            class: 2,
            issued_at: Some(0),
            expires_at: Some(10),
            reference: None,
            reference_hash: None,
        },
        TokenMetadata {
            class: 1,
            issued_at: Some(0),
            expires_at: Some(10),
            reference: None,
            reference_hash: None,
        },
    ];

    let token_spec = vec![
        (alice_acc.id(), alice_tokens),
        (bob_acc.id(), bob_tokens),
        (john_acc.id(), john_tokens),
        (elon_acc.id(), elon_tokens),
    ];

    let res = iah_issuer
        .call(registry_contract.id(), "sbt_mint")
        .args_json(json!({ "token_spec": token_spec }))
        .deposit(parse_near!("1 N"))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    return Ok((
        ndc_nominations_contract,
        alice_acc,
        bob_acc,
        john_acc,
        elon_acc,
    ));
}

#[tokio::test]
async fn self_nominate() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, alice, _, _, _) = init(&worker).await?;

    // slef nominate
    let res = alice
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    println!("Passed ✅ self_nominate");
    Ok(())
}

#[tokio::test]
async fn self_nominate_only_og() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, _, _, john, _) = init(&worker).await?;

    // slef nominate
    let res = john
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    println!("Passed ✅ self_nominate_only_og");
    Ok(())
}

#[tokio::test]
async fn self_nominate_only_iah_fail() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, _, bob, _, _) = init(&worker).await?;

    // slef nominate
    let res = bob
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await;
    assert!(format!("{:?}", res).contains("Not a verified OG member, or the token is expired"));

    println!("Passed ✅ self_nominate_only_iah_fail");
    Ok(())
}

#[tokio::test]
async fn self_nominate_expired_token_fail() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, _, _, _, elon) = init(&worker).await?;

    // self nominate
    let res = elon
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await;
    assert!(format!("{:?}", res).contains("Not a verified OG member, or the token is expired"));

    println!("Passed ✅ self_nominate_expired_token_fail");
    Ok(())
}

#[tokio::test]
async fn upvote() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, _, bob, john, _) = init(&worker).await?;

    // self nominate
    let res = john
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // upvote johns nomination
    let res = bob
        .call(ndc_elections_contract.id(), "upvote")
        .args_json(json!({"candidate": john.id(),}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    println!("Passed ✅ upvote");
    Ok(())
}

#[tokio::test]
async fn double_upvote_fail() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, _, bob, john, _) = init(&worker).await?;

    // self nominate
    let res = john
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // upvote johns nomination
    let res = bob
        .call(ndc_elections_contract.id(), "upvote")
        .args_json(json!({"candidate": john.id(),}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // double upvote
    let res = bob
        .call(ndc_elections_contract.id(), "upvote")
        .args_json(json!({"candidate": john.id(),}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await;
    assert!(format!("{:?}", res).contains("Nomination already upvoted"));

    println!("Passed ✅ double_upvote_fail");
    Ok(())
}

#[tokio::test]
async fn upvote_by_non_human_fail() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, alice, _, john, _) = init(&worker).await?;

    // self nominate
    let res = alice
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // john not iah upvotes alice nomination
    let res = john
        .call(ndc_elections_contract.id(), "upvote")
        .args_json(json!({"candidate": alice.id(),}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await;
    assert!(format!("{:?}", res).contains("Not a verified human member, or the tokens are expired"));

    println!("Passed ✅ upvote_by_non_human");
    Ok(())
}

#[tokio::test]
async fn upvote_expired_iah_fail() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, alice, _, _, elon) = init(&worker).await?;

    // self nominate
    let res = alice
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // john not iah upvotes alice nomination
    let res = elon
        .call(ndc_elections_contract.id(), "upvote")
        .args_json(json!({"candidate": alice.id(),}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await;
    assert!(format!("{:?}", res).contains("Not a verified human member, or the tokens are expired"));

    println!("Passed ✅ upvote_by_non_human");
    Ok(())
}

#[tokio::test]
async fn comment() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, _, bob, john, _) = init(&worker).await?;

    // self nominate
    let res = john
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // upvote johns nomination
    let res = bob
        .call(ndc_elections_contract.id(), "comment")
        .args_json(json!({"candidate": john.id(), "comment": "solid candidate",}))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    println!("Passed ✅ comment ");
    Ok(())
}

#[tokio::test]
async fn comment_by_non_human_fail() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, alice, _, john, _) = init(&worker).await?;

    // self nominate
    let res = alice
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // john not iah upvotes alice nomination
    let res = john
        .call(ndc_elections_contract.id(), "comment")
        .args_json(json!({"candidate": alice.id(),"comment": "solid candidate"}))
        .max_gas()
        .transact()
        .await;
    assert!(format!("{:?}", res).contains("Not a verified human member, or the tokens are expired"));

    println!("Passed ✅ comment_by_non_human");
    Ok(())
}

#[tokio::test]
async fn comment_expired_iah_fail() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, alice, _, _, elon) = init(&worker).await?;

    // self nominate
    let res = alice
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // john not iah upvotes alice nomination
    let res = elon
        .call(ndc_elections_contract.id(), "comment")
        .args_json(json!({"candidate": alice.id(),"comment": "solid candidate"}))
        .max_gas()
        .transact()
        .await;
    assert!(format!("{:?}", res).contains("Not a verified human member, or the tokens are expired"));

    println!("Passed ✅ comment_expired_iah_fail");
    Ok(())
}

#[tokio::test]
async fn flow1() -> anyhow::Result<()> {
    let worker = workspaces::sandbox().await?;
    let (ndc_elections_contract, _, bob, john, _) = init(&worker).await?;

    // self nominate
    let res = john
        .call(ndc_elections_contract.id(), "self_nominate")
        .args_json(json!({"house": HouseType::HouseOfMerit, "comment": "solid nomination", "link": "external_link.io"}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // upvote johns nomination
    let res = bob
        .call(ndc_elections_contract.id(), "upvote")
        .args_json(json!({"candidate": john.id(),}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    // self revoke
    let res = john
        .call(ndc_elections_contract.id(), "self_revoke")
        .args_json(json!({}))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    // self nominate to a different house
    let res = john
     .call(ndc_elections_contract.id(), "self_nominate")
     .args_json(json!({"house": HouseType::CouncilOfAdvisors, "comment": "solid nomination", "link": "external_link.io"}))
     .max_gas()
     .deposit(parse_near!("1 N"))
     .transact()
     .await?;
    assert!(res.is_success());

    // upvote johns new nomination
    let res = bob
        .call(ndc_elections_contract.id(), "upvote")
        .args_json(json!({"candidate": john.id(),}))
        .max_gas()
        .deposit(parse_near!("1 N"))
        .transact()
        .await?;
    assert!(res.is_success());

    println!("Passed ✅ flow1");
    Ok(())
}

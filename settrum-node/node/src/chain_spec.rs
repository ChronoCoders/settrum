use sc_service::ChainType;
use settrum_runtime::{AccountId, AuraId, GrandpaId};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

pub type ChainSpec = sc_service::GenericChainSpec;

type AccountPublic = <settrum_runtime::Signature as Verify>::Signer;

fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{seed}"), None)
        .expect("static values are valid; qed")
        .public()
}

fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

fn authority_keys_from_seed(seed: &str) -> (AccountId, AuraId, GrandpaId) {
    (
        get_account_id_from_seed::<sr25519::Public>(seed),
        get_from_seed::<AuraId>(seed),
        get_from_seed::<GrandpaId>(seed),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
    Ok(
        ChainSpec::builder(settrum_runtime::WASM_BINARY.unwrap_or_default(), None)
            .with_name("Development")
            .with_id("dev")
            .with_chain_type(ChainType::Development)
            .with_genesis_config_patch(testnet_genesis(
                vec![authority_keys_from_seed("Alice")],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                ],
            ))
            .build(),
    )
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    Ok(
        ChainSpec::builder(settrum_runtime::WASM_BINARY.unwrap_or_default(), None)
            .with_name("Local Testnet")
            .with_id("local_testnet")
            .with_chain_type(ChainType::Local)
            .with_genesis_config_patch(testnet_genesis(
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
                ],
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                ],
            ))
            .build(),
    )
}

fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
    sudo_key: AccountId,
    _endowed_accounts: Vec<AccountId>,
) -> serde_json::Value {
    let aura_authorities: Vec<(AuraId,)> = initial_authorities
        .iter()
        .map(|(_, aura, _)| (aura.clone(),))
        .collect();

    let grandpa_authorities: Vec<(GrandpaId, u64)> = initial_authorities
        .iter()
        .map(|(_, _, grandpa)| (grandpa.clone(), 1u64))
        .collect();

    serde_json::json!({
        "system": {},
        "sudo": {
            "key": Some(sudo_key)
        },
        "aura": {
            "authorities": aura_authorities.iter().map(|(a,)| a).collect::<Vec<_>>()
        },
        "grandpa": {
            "authorities": grandpa_authorities
        },
    })
}

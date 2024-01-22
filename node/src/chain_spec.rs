use node_template_runtime::{
	AccountId, AuraConfig, BalancesConfig, RuntimeGenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY, NodeAuthorizationConfig, opaque::SessionKeys, ValidatorSetConfig, SessionConfig
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public, OpaquePeerId};
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
//pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
//	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
//}


pub fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
	SessionKeys { aura, grandpa }
}

pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s)
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				// following should be used for PRODUCTION
				// This is a vector of (AccountId, AuraId, GrandpaID) tuples
				// AccountId and AuraId are derived from the Sr25519 public key string in hex without 0x prefix
				// GrandpaId is derived from the Ed25519 public key string in hex without 0x prefix 
				// Note the different functions used for conversion
				// Replace each key based on custom key used
				/*vec![
				(
					array_bytes::hex_n_into_unchecked("98ac86a111826e4176916ae81c7443075138b9a69760dd4536d06f8a16f2501f"),
					array_bytes::hex2array_unchecked("98ac86a111826e4176916ae81c7443075138b9a69760dd4536d06f8a16f2501f").unchecked_into(),
					array_bytes::hex2array_unchecked("1ce5f00ef6e89374afb625f1ae4c1546d31234e87e3c3f51a62b91dd6bfa57df").unchecked_into(),
				),
				(
					array_bytes::hex_n_into_unchecked("cc056f8d99996a960227488332f015ce988a809648dc1591bb124570cd367536"),
					array_bytes::hex2array_unchecked("cc056f8d99996a960227488332f015ce988a809648dc1591bb124570cd367536").unchecked_into(),
					array_bytes::hex2array_unchecked("dacde7714d8551f674b8bb4b54239383c76a2b286fa436e93b2b7eb226bf4de7").unchecked_into(),
				)
				],*/
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				// following should be used for PRODUCTION
				// This is the AccountId derived from Sr25519 public key string in hex without 0x prefix
				/*array_bytes::hex_n_into_unchecked("98ac86a111826e4176916ae81c7443075138b9a69760dd4536d06f8a16f2501f"),*/
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				// include only following for PRODUCTION
				// Each element is an AccountId derived from Sr25519 public key string in hex without 0x prefix 
				/*vec![
					array_bytes::hex_n_into_unchecked("98ac86a111826e4176916ae81c7443075138b9a69760dd4536d06f8a16f2501f"),
					array_bytes::hex_n_into_unchecked("cc056f8d99996a960227488332f015ce988a809648dc1591bb124570cd367536"),
				],*/
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("sync"),
					get_account_id_from_seed::<sr25519::Public>("zkxnode"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("sync//stash"),
					get_account_id_from_seed::<sr25519::Public>("zkxnode//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
		system: SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		validator_set: ValidatorSetConfig {
			initial_validators: initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
		},
		// session accounts must have some balance
		// keys are set here, the initial validator set is provided by the substrate_validator_set pallet
		session: SessionConfig {
			keys: initial_authorities.iter().map(|x| {
				(x.0.clone(), x.0.clone(), session_keys(x.1.clone(), x.2.clone()))
			}).collect::<Vec<_>>(),
		},
		// since validators are provided by the session pallet, they are not initialized separately
		// for aura and grandpa pallets
		aura: AuraConfig {
			authorities: vec![],
		},
		grandpa: GrandpaConfig {
			authorities: vec![],
			..Default::default()
		},
		sudo: SudoConfig {
			// Assign network admin rights.
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
		node_authorization: NodeAuthorizationConfig {
			// nodes is a vector of tuples
			// first element is the PeerID
			// 2nd element is the accountID
			// this defines the initial set of authorized nodes at genesis
			// For PRODUCTION change the peerId based on custom keys used
			nodes: vec![
				(
				OpaquePeerId(bs58::decode("12D3KooWBmAwcd4PJNJvfV89HwE48nwkRmAgo8Vy3uQEyNNHBox2").into_vec().unwrap()),
				endowed_accounts[0].clone()
				),
				(
				OpaquePeerId(bs58::decode("12D3KooWQYV9dGMFoRzNStwpXztXaBUjtPqi6aU76ZgUriHhKust").into_vec().unwrap()),
				endowed_accounts[1].clone()
				),
			],
 		},
	}
}

use schemars::JsonSchema;
use serde::{ Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Addr}; 
use secret_toolkit::{ 
    storage:: { Item, Keymap, AppendStore }
};
use crate::msg::{HistoryToken, ContractInfo, Level, Token};

pub static CONFIG_KEY: &[u8] = b"config"; 
pub const ADMIN_KEY: &[u8] = b"admin";
pub const MY_ADDRESS_KEY: &[u8] = b"my_address"; 
pub const INHOLDING_NFT_KEY: &[u8] = b"inholding_nft";
pub const PREFIX_REVOKED_PERMITS: &str = "revoke";
pub const LEVEL_KEY: &[u8] = b"level";
pub const HISTORY_KEY: &[u8] = b"level";

pub static CONFIG_ITEM: Item<State> = Item::new(CONFIG_KEY); 
pub static LEVEL_ITEM: Item<Vec<Level>> = Item::new(LEVEL_KEY);
pub static ADMIN_ITEM: Item<CanonicalAddr> = Item::new(ADMIN_KEY); 
pub static MY_ADDRESS_ITEM: Item<CanonicalAddr> = Item::new(MY_ADDRESS_KEY);  
pub static INHOLDING_NFT_STORE: Keymap<CanonicalAddr, Token> = Keymap::new(INHOLDING_NFT_KEY);
pub static HISTORY_STORE: AppendStore<HistoryToken> = AppendStore::new(HISTORY_KEY);


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {  
    pub owner: Addr,   
    pub wolf_pack_contract: ContractInfo,
    pub power_up_contract: ContractInfo,
    pub level_cap: i32, 
    pub viewing_key: Option<String>,
    pub total_power_ups: i32,
    pub total_xp_boost: i32,
    pub total_bones_used: i32
} 
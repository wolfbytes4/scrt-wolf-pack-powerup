use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{
   Addr, Binary
};
use secret_toolkit::{ 
    permit:: { Permit }
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {  
    pub entropy: String,
    pub wolf_pack_contract: ContractInfo,
    pub power_up_contract: ContractInfo, 
    pub level_cap: i32,
    pub levels: Vec<Level>
} 

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractInfo {
    /// contract's code hash string
    pub code_hash: String,
    /// contract's address
    pub address: Addr
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Level {
    pub level: i32,
    pub xp_needed: i32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PowerUpMsg {
    pub wolf_token_id: String,
    pub powerup_token_ids: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct HistoryToken {
    pub wolf_token_id: String,
    pub powerup_token_ids: Vec<String>, 
    pub power_up_date: Option<u64>, 
    pub power_up_amount: i32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct RelatedTokensMsg {
    pub related_token_ids: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Token {
    pub token_id: String,
    pub owner: Addr,
    pub sender: Addr,
    pub related_token_ids: Vec<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PowerUpInfoResponse {
    pub total_power_ups: i32,
    pub total_xp_boost: i32,
    pub total_bones_used: i32
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg { 
    RevokePermit{
        permit_name: String
    }, 
    BatchReceiveNft{
        from: Addr, 
        token_ids: Vec<String>,
        msg: Option<Binary>
    },
    SendNftBack{ 
        token_id: String,
        owner: Addr
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {   
    GetPowerUpInfo {},
    GetUserHistory { 
        permit: Permit,
        start_page: u32, 
        page_size: u32 
    },
    GetNumUserHistory{
        permit: Permit
    }
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ContractsResponse {
    pub contract_infos: Vec<ContractInfo>,
}
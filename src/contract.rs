use cosmwasm_std::{
    entry_point, to_binary, Env, Deps, DepsMut,
    MessageInfo, Response, StdError, StdResult, Addr, CanonicalAddr,
    Binary, CosmosMsg
};
use crate::error::ContractError;
use crate::msg::{PowerUpInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg, HistoryToken, RelatedTokensMsg, Token };
use crate::state::{ State, CONFIG_ITEM, LEVEL_ITEM, ADMIN_ITEM, INHOLDING_NFT_STORE, MY_ADDRESS_ITEM, PREFIX_REVOKED_PERMITS, HISTORY_STORE};
use crate::rand::{sha_256};
use secret_toolkit::{
    snip721::{
        batch_burn_nft_msg, register_receive_nft_msg, set_viewing_key_msg, nft_dossier_query, transfer_nft_msg, set_metadata_msg, ViewerInfo, Metadata, NftDossier, Burn
    },
    permit::{validate, Permit, RevokedPermits}
};  
pub const BLOCK_SIZE: usize = 256;


#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, StdError> {
    let prng_seed: Vec<u8> = sha_256(base64::encode(msg.entropy).as_bytes()).to_vec();
    let viewing_key = base64::encode(&prng_seed);
    // TODO
    // NEED to register receive on SHILL and SCRT(done)
    // add viewing key for scrt(done)
    // add leveling chart up to 100
    // add bone chart with types and xp

    // create initial state
    let state = State { 
        viewing_key: Some(viewing_key),
        owner: info.sender.clone(),  
        wolf_pack_contract: msg.wolf_pack_contract,
        power_up_contract: msg.power_up_contract,
        level_cap: msg.level_cap,
        total_bones_used: 0,
        total_power_ups: 0,
        total_xp_boost: 0
    };

    //Save Contract state
    CONFIG_ITEM.save(deps.storage, &state)?;
    LEVEL_ITEM.save(deps.storage, &msg.levels)?;
    ADMIN_ITEM.save(deps.storage, &deps.api.addr_canonicalize(&info.sender.to_string())?)?;
    MY_ADDRESS_ITEM.save(deps.storage,  &deps.api.addr_canonicalize(&_env.contract.address.to_string())?)?;
 
 
   
    deps.api.debug(&format!("Contract was initialized by {}", info.sender));
     
    let vk = state.viewing_key.unwrap();
    Ok(Response::new()
        .add_message(register_receive_nft_msg(
            _env.contract.code_hash.clone(),
            Some(true),
            None,
            BLOCK_SIZE,
            state.wolf_pack_contract.code_hash.clone(),
            state.wolf_pack_contract.address.clone().to_string(),
        )?)
        .add_message(set_viewing_key_msg(
            vk.to_string(),
            None,
            BLOCK_SIZE,
            state.wolf_pack_contract.code_hash,
            state.wolf_pack_contract.address.to_string(),
        )?) 
        .add_message(register_receive_nft_msg(
            _env.contract.code_hash.clone(),
            Some(true),
            None,
            BLOCK_SIZE,
            state.power_up_contract.code_hash.clone(),
            state.power_up_contract.address.clone().to_string(),
        )?)
        .add_message(set_viewing_key_msg(
            vk.to_string(),
            None,
            BLOCK_SIZE,
            state.power_up_contract.code_hash,
            state.power_up_contract.address.to_string(),
        )?) 
    )
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg
) -> Result<Response, ContractError> {
    match msg { 
        ExecuteMsg::RevokePermit { permit_name } => {
            try_revoke_permit(deps, &info.sender, &permit_name)
        },
        ExecuteMsg::BatchReceiveNft { from, token_ids, msg } => {
            try_batch_receive(deps, _env, &info.sender, &from, token_ids, msg)
        },
        ExecuteMsg::SendNftBack { token_id, owner } => {
            try_send_nft_back(deps, _env, &info.sender, token_id, owner)
        }
    }
} 

fn try_batch_receive(
    deps: DepsMut,
    _env: Env,
    sender: &Addr,
    from: &Addr,
    token_ids: Vec<String>,
    msg: Option<Binary>,
) -> Result<Response, ContractError> { 
    deps.api.debug(&format!("Batch received"));
    //get bone type and translate to xp
    let mut response_msgs: Vec<CosmosMsg> = Vec::new();
    let mut response_attrs = vec![];
    let mut state = CONFIG_ITEM.load(deps.storage)?; 
    let levels = LEVEL_ITEM.load(deps.storage)?;
    let inholding_nft: Token = INHOLDING_NFT_STORE.get(deps.storage, &deps.api.addr_canonicalize(&from.to_string())?)
                                                       .unwrap_or(Token {owner: Addr::unchecked(""), sender: Addr::unchecked(""), token_id: "".to_string(), related_token_ids: Vec::new()});

    //update nft's lvl and xp 
    if let Some(bin) = msg { 
     let bytes = base64::decode(bin.to_base64()).unwrap();
     let pmsg: RelatedTokensMsg = serde_json::from_slice(&bytes).unwrap();

          
     if sender == &state.wolf_pack_contract.address{
         if inholding_nft.owner.to_string().len() > 0  {
            return Err(ContractError::CustomError {val: "You already have a wolf in holding for power up".to_string()});
         }
         if token_ids.len() > 1{
            return Err(ContractError::CustomError {val: "You can only power up one wolf at a time".to_string()});
         }
         if pmsg.related_token_ids.len() == 0{ 
            return Err(ContractError::CustomError {val: "No power up selected".to_string()});
         }
         let inholding_wolf = Token {
             token_id: token_ids[0].to_string(),
             owner: from.clone(),
             sender: sender.clone(),
             related_token_ids: pmsg.related_token_ids
         };
           //transfer back 

         INHOLDING_NFT_STORE.insert(deps.storage, &deps.api.addr_canonicalize(&from.to_string())?, &inholding_wolf)?;
     } 
 
     else if sender == &state.power_up_contract.address{
        let history_store = HISTORY_STORE.add_suffix(from.to_string().as_bytes());
        if inholding_nft.owner.to_string().len() == 0 {
            return Err(ContractError::CustomError {val: "You do not have a wolf to power up :(".to_string()});
        }
        // Get viewing key for NFTs
        let viewer = Some(ViewerInfo {
            address: _env.contract.address.to_string(),
            viewing_key: state.viewing_key.as_ref().unwrap().to_string(),
        });
        
        let mut xp_boost: i32 = 0;
        for token_id in token_ids.iter() { 
            let meta: NftDossier =  nft_dossier_query(
                deps.querier,
                token_id.to_string(),
                viewer.clone(),
                None,
                BLOCK_SIZE,
                state.power_up_contract.code_hash.clone(),
                state.power_up_contract.address.to_string(),
            )?;
            if let Some(Metadata { extension, .. }) = meta.public_metadata {
                if let Some(mut ext) = extension { 
                    let xp_boost_trait = ext.attributes.as_ref().unwrap().iter().find(|&x| x.trait_type == Some("XP Boost".to_string())).unwrap();
                    xp_boost += xp_boost_trait.value.parse::<i32>().unwrap();
                }
            } 
        }


        let wolf_meta: NftDossier =  nft_dossier_query(
            deps.querier,
            inholding_nft.token_id.to_string(),
            viewer.clone(),
            None,
            BLOCK_SIZE,
            state.wolf_pack_contract.code_hash.clone(),
            state.wolf_pack_contract.address.to_string(),
        )?;

        let new_ext = 
                if let Some(Metadata { extension, .. }) = wolf_meta.public_metadata {
                    if let Some(mut ext) = extension { 
                        let current_xp_trait = ext.attributes.as_ref().unwrap().iter().find(|&x| x.trait_type == Some("XP".to_string())).unwrap();
                        let current_xp = current_xp_trait.value.parse::<i32>().unwrap() + xp_boost;
                        let current_lvl_trait = ext.attributes.as_ref().unwrap().iter().find(|&x| x.trait_type == Some("LVL".to_string())).unwrap();
                        let current_lvl = current_lvl_trait.value.parse::<i32>().unwrap();
                        for attr in ext.attributes.as_mut().unwrap().iter_mut() {

                            if attr.trait_type == Some("XP".to_string()) {
                                attr.value = current_xp.to_string();
                            }  

                            if attr.trait_type == Some("LVL".to_string()) {
                                let shouldbe_lvl = if attr.value.parse::<i32>().unwrap() < state.level_cap {
                                        levels.iter().find(|&x| x.xp_needed > current_xp).unwrap().level - 1
                                    } 
                                    else { 
                                        attr.value.parse::<i32>().unwrap() 
                                    }; 
                                attr.value = shouldbe_lvl.to_string(); 

                                if shouldbe_lvl > current_lvl {
                                    response_attrs.push(("lvl_increase".to_string(), shouldbe_lvl.to_string()));
                                }
                            }  
                        }
                        ext 
                    }
                    else {
                        return Err(ContractError::CustomError {val: "unable to set metadata with uri".to_string()});
                    }
                } 
                else {
                    return Err(ContractError::CustomError {val: "unable to get metadata from nft contract".to_string()});
                };

        //add bone burn to responses
        let mut burns: Vec<Burn> = Vec::new(); 
        burns.push(
            Burn{ 
                token_ids: token_ids.clone(),
                memo: None
            }
        );

        let cosmos_batch_msg = batch_burn_nft_msg(
            burns,
            None,
            BLOCK_SIZE,
            state.power_up_contract.code_hash.clone(),
            state.power_up_contract.address.to_string(),
        )?;
        response_msgs.push(cosmos_batch_msg);  
        
        //add metadata update to responses
        let cosmos_msg = set_metadata_msg(
            inholding_nft.token_id.to_string(),
            Some(Metadata {
                token_uri: None,
                extension: Some(new_ext),
            }),
            None,
            None,
            BLOCK_SIZE,
            state.wolf_pack_contract.code_hash.clone(),
            state.wolf_pack_contract.address.to_string()
        )?;
        response_msgs.push(cosmos_msg); 

        // ensure that the NFT exists and is owned by the contract
        if wolf_meta.owner.unwrap() != _env.contract.address.to_string() {
            return Err(ContractError::CustomError {val: "Wolf not owned by contract".to_string()}); 
        }

        response_attrs.push(("xp_boost_amount".to_string(), xp_boost.to_string()));

        // add transfer update to responses
        let cosmos_transfer_msg = transfer_nft_msg(
            inholding_nft.owner.to_string(),
            inholding_nft.token_id.to_string(),
            None,
            None,
            BLOCK_SIZE,
            state.wolf_pack_contract.code_hash.to_string(),
            state.wolf_pack_contract.address.to_string()
        )?;
        response_msgs.push(cosmos_transfer_msg);
        //enter history record
        let history_token: HistoryToken = { HistoryToken {
            wolf_token_id: inholding_nft.token_id.to_string(),
            powerup_token_ids: token_ids.clone(),
            power_up_date: Some(_env.block.time.seconds()), 
            power_up_amount: xp_boost
        }};
        
        history_store.push(deps.storage, &history_token)?;

        //update state
        state.total_power_ups = state.total_power_ups + 1;
        state.total_xp_boost = state.total_xp_boost + xp_boost;
        state.total_bones_used = state.total_bones_used + token_ids.len()as i32;
        
        CONFIG_ITEM.save(deps.storage, &state)?;
        //remove nft from inholding store
        INHOLDING_NFT_STORE.remove(deps.storage, &deps.api.addr_canonicalize(&from.to_string())?)?;
     }
     else{
        return Err(ContractError::CustomError {val: "Not a valid contract address".to_string()});
     }

       
     }
   
   else{
    return Err(ContractError::CustomError {val: "Invalid message received".to_string()});
   }   

 
   Ok(Response::new().add_messages(response_msgs).add_attributes(response_attrs))
    
}


pub fn try_send_nft_back(
    deps: DepsMut,
    _env: Env,
    sender: &Addr,
    token_id: String,
    owner: Addr
) -> Result<Response, ContractError> { 
    let mut nft = Token{ owner: Addr::unchecked(""), sender: Addr::unchecked(""), token_id: "".to_string(), related_token_ids: Vec::new()};
    let mut contract: Option<String> = None;
    let mut hash: Option<String> = None;

    let state = CONFIG_ITEM.load(deps.storage)?;
    let inholding_nft: Token = INHOLDING_NFT_STORE.get(deps.storage,&deps.api.addr_canonicalize(&owner.to_string())?).unwrap_or(Token{ owner: Addr::unchecked(""), sender: Addr::unchecked(""), token_id: "".to_string(), related_token_ids: Vec::new()});
    if inholding_nft.owner.to_string().len() == 0
    {
        return Err(ContractError::CustomError {val: "This address does not have anything staked".to_string()});
    }

        if sender.clone() != state.owner {
            return Err(ContractError::CustomError {val: "You don't have the permissions to execute this command".to_string()});
        }  
        if inholding_nft.token_id == token_id {
            nft = inholding_nft.clone();
            hash = Some(state.wolf_pack_contract.code_hash.to_string());
            contract = Some(state.wolf_pack_contract.address.to_string());
        }
        else{
            return Err(ContractError::CustomError {val: "Token doesn't exist".to_string()});
        }
          
        INHOLDING_NFT_STORE.remove(deps.storage, &deps.api.addr_canonicalize(&owner.to_string())?)?;
    Ok(Response::new()
        .add_message(transfer_nft_msg(
            nft.owner.to_string(),
            nft.token_id.to_string(),
            None,
            None,
            BLOCK_SIZE,
            hash.unwrap().to_string(),
            contract.unwrap().to_string()
        )?)
    )
}
fn try_revoke_permit(
    deps: DepsMut,
    sender: &Addr,
    permit_name: &str,
) -> Result<Response, ContractError> {
    RevokedPermits::revoke_permit(deps.storage, PREFIX_REVOKED_PERMITS, &sender.to_string(), permit_name);
    
    Ok(Response::default())
}

#[entry_point]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {   
        QueryMsg::GetPowerUpInfo {} => to_binary(&query_power_up_info(deps)?),  
        QueryMsg::GetNumUserHistory { permit } => to_binary(&query_num_user_history(deps, permit)?),
        QueryMsg::GetUserHistory {permit, start_page, page_size} => to_binary(&query_user_history(deps, permit, start_page, page_size)?),
    }
}

fn query_user_history(
    deps: Deps, 
    permit: Permit,
    start_page: u32, 
    page_size: u32
) -> StdResult<Vec<HistoryToken>> {
    let (user_raw, my_addr) = get_querier(deps, permit)?;
    
    let history_store = HISTORY_STORE.add_suffix(&user_raw); 
    let history = history_store.paging(deps.storage, start_page, page_size)?;
    Ok(history)
}

fn query_power_up_info(
    deps: Deps,
) -> StdResult<PowerUpInfoResponse> { 
    let state = CONFIG_ITEM.load(deps.storage)?;

    Ok(PowerUpInfoResponse { total_bones_used: state.total_bones_used, total_power_ups: state.total_power_ups, total_xp_boost: state.total_xp_boost })
} 
 
fn query_num_user_history(
    deps: Deps, 
    permit: Permit
) -> StdResult<u32> { 
    let (user_raw, my_addr) = get_querier(deps, permit)?;
    let history_store = HISTORY_STORE.add_suffix(&user_raw);
    let num = history_store.get_len(deps.storage)?;
    Ok(num)
}  

fn get_querier(
    deps: Deps,
    permit: Permit,
) -> StdResult<(CanonicalAddr, Option<CanonicalAddr>)> {
    if let pmt = permit {
        let me_raw: CanonicalAddr = MY_ADDRESS_ITEM.load(deps.storage)?;
        let my_address = deps.api.addr_humanize(&me_raw)?;
        let querier = deps.api.addr_canonicalize(&validate(
            deps,
            PREFIX_REVOKED_PERMITS,
            &pmt,
            my_address.to_string(),
            None
        )?)?;
        if !pmt.check_permission(&secret_toolkit::permit::TokenPermissions::Owner) {
            return Err(StdError::generic_err(format!(
                "Owner permission is required for history queries, got permissions {:?}",
                pmt.params.permissions
            )));
        }
        return Ok((querier, Some(me_raw)));
    }
    return Err(StdError::generic_err(
        "Unauthorized",
    ));  
}


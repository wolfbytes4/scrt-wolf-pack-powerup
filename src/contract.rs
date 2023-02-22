use cosmwasm_std::{
    entry_point, to_binary, Env, Deps, DepsMut,
    MessageInfo, Response, StdError, StdResult, Addr, CanonicalAddr,
    Binary, Uint128, CosmosMsg
};
use crate::error::ContractError;
use crate::msg::{ContractsResponse, BurnInfoResponse, ExecuteMsg, InstantiateMsg, QueryMsg, ContractInfo, HistoryToken };
use crate::state::{ State, CONFIG_ITEM, ADMIN_ITEM, BURN_HISTORY_STORE, MY_ADDRESS_ITEM, PREFIX_REVOKED_PERMITS};
use secret_toolkit::{
    snip721::{
        batch_burn_nft_msg, register_receive_nft_msg, set_viewing_key_msg, Burn
    },
    permit::{validate, Permit, RevokedPermits},
    snip20::{ transfer_msg }
};  
pub const BLOCK_SIZE: usize = 256;


#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg
) -> Result<Response, StdError> {
    // TODO
    // NEED to register receive on SHILL and SCRT
    // add viewing key for scrt
    // add leveling chart up to 100
    // add bone chart with types and xp

    // create initial state
    let state = State { 
        owner: info.sender.clone(),  
        contract_infos: msg.contract_infos,
        shill_contract: msg.shill_contract,
        shill_viewing_key: Some(msg.entropy_shill),
        amount_paid: Uint128::from(0u32),
        num_burned: 0
    };

    //Save Contract state
    CONFIG_ITEM.save(deps.storage, &state)?;
    ADMIN_ITEM.save(deps.storage, &deps.api.addr_canonicalize(&info.sender.to_string())?)?;
    MY_ADDRESS_ITEM.save(deps.storage,  &deps.api.addr_canonicalize(&_env.contract.address.to_string())?)?;
 
    let mut response_msgs: Vec<CosmosMsg> = Vec::new();
    for contract_info in state.contract_infos.iter() { 
        response_msgs.push(
            register_receive_nft_msg(
                _env.contract.code_hash.clone(),
                Some(true),
                None,
                BLOCK_SIZE,
                contract_info.code_hash.clone(),
                contract_info.address.clone().to_string(),
            )?
        ); 
    }
   
    response_msgs.push(
        set_viewing_key_msg(
            state.shill_viewing_key.unwrap().to_string(),
            None,
            BLOCK_SIZE,
            state.shill_contract.code_hash,
            state.shill_contract.address.to_string(),
        )?
    );
   
    deps.api.debug(&format!("Contract was initialized by {}", info.sender));
     
    Ok(Response::new().add_messages(response_msgs))
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

        ExecuteMsg::Receive { from, amount, msg } => try_receive(deps, env, &info.sender, from, amount, msg),
        ExecuteMsg::SendScrt { amount, address } => {
            try_send_scrt(deps, _env, &info.sender, amount, address)
        },
        ExecuteMsg::SendShill { amount, address } => {
            try_send_shill(deps, _env, &info.sender, amount, address)
        }
    }
} 

fn try_receive(
    deps: DepsMut,
    _env: Env,
    sender: &Addr,
    from: &Addr,
    amount: Uint128,
    msg: Option<Binary>
)-> Result<Response, ContractError> { 
    //check that token is acceptable one

    //check if amount is correct for either $Shill or $sScrt

    //use permit and check if wolf token id and the bone token id is owned by &from
    
    //get bone type and translate to xp

    //update nft's lvl and xp 
}



fn try_revoke_permit(
    deps: DepsMut,
    sender: &Addr,
    permit_name: &str,
) -> Result<Response, ContractError> {
    RevokedPermits::revoke_permit(deps.storage, PREFIX_REVOKED_PERMITS, &sender.to_string(), permit_name);
    
    Ok(Response::default())
}

pub fn try_send_scrt(
    deps: DepsMut,
    _env: Env,
    sender: &Addr,
    amount: Uint128,
    address: Addr
) -> Result<Response, ContractError> {  
    let state = CONFIG_ITEM.load(deps.storage)?;
    if sender.clone() != state.owner {
        return Err(ContractError::Unauthorized {});
    }
   
    Ok(Response::new().add_message(
        transfer_msg(
            address.to_string(),
            amount,
            None,
            None,
            256,
            state.shill_contract.code_hash.to_string(),
            state.shill_contract.address.to_string()
        )?)
    ) 
}

pub fn try_send_shill(
    deps: DepsMut,
    _env: Env,
    sender: &Addr,
    amount: Uint128,
    address: Addr
) -> Result<Response, ContractError> {  
    let state = CONFIG_ITEM.load(deps.storage)?;
    if sender.clone() != state.owner {
        return Err(ContractError::Unauthorized {});
    }
   
    Ok(Response::new().add_message(
        transfer_msg(
            address.to_string(),
            amount,
            None,
            None,
            256,
            state.shill_contract.code_hash.to_string(),
            state.shill_contract.address.to_string()
        )?)
    ) 
}

#[entry_point]
pub fn query(
    deps: Deps,
    _env: Env,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {   
        QueryMsg::GetNumLiteHistory { } => to_binary(&query_num_lite_history(deps)?),
        QueryMsg::GetLiteHistory {start_page, page_size} => to_binary(&query_lite_history(deps, start_page, page_size)?),
        QueryMsg::GetNumUserHistory { permit } => to_binary(&query_num_user_history(deps, permit)?),
        QueryMsg::GetUserHistory {permit, start_page, page_size} => to_binary(&query_user_history(deps, permit, start_page, page_size)?),
    }
}
 
fn query_lite_history(
    deps: Deps,  
    start_page: u32, 
    page_size: u32
) -> StdResult<Vec<HistoryToken>> { 
    Ok()
}

fn query_num_lite_history(
    deps: Deps
) -> StdResult<u32> { 
    // let (user_raw, my_addr) = get_querier(deps, permit)?;
    // let burn_history_store = BURN_HISTORY_STORE.add_suffix(&user_raw);
    // let num = burn_history_store.get_len(deps.storage)?;
    Ok()
}   

fn query_user_history(
    deps: Deps, 
    permit: Permit,
    start_page: u32, 
    page_size: u32
) -> StdResult<Vec<HistoryToken>> {
    let (user_raw, my_addr) = get_querier(deps, permit)?;
    
    let burn_history_store = BURN_HISTORY_STORE.add_suffix(&user_raw); 
    let history = burn_history_store.paging(deps.storage, start_page, page_size)?;
    Ok(history)
}

fn query_num_user_history(
    deps: Deps, 
    permit: Permit
) -> StdResult<u32> { 
    let (user_raw, my_addr) = get_querier(deps, permit)?;
    let burn_history_store = BURN_HISTORY_STORE.add_suffix(&user_raw);
    let num = burn_history_store.get_len(deps.storage)?;
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


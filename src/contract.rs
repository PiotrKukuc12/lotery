#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ResultAdmin, ResultGameInfo};
use crate::state::{Admin, GameState, Player, Status, ADMIN, GAMESTATE};

use rand::Rng;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:lotery";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let admin = Admin {
        owner: info.sender.clone(),
    };
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    ADMIN.save(deps.storage, &admin)?;

    let init_game = GameState {
        participants: Vec::new(),
        random_number: Some(0),
        status: Some(Status::Finished {}),
        num_diff: 0,
        winner: None,
    };

    GAMESTATE.save(deps.storage, &init_game)?;

    Ok(Response::new()
        .add_attribute("method", "instantiate")
        .add_attribute("owner", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::ChooseNumber { number } => try_choose_number(deps, info, number),
        ExecuteMsg::ResetGame {} => try_reset_game(deps, info),
        ExecuteMsg::EndGame {} => try_end_game(deps, info),
    }
}

pub fn try_end_game(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;

    if info.sender != admin.owner {
        return Err(ContractError::Unauthorized {});
    }

    let game = GAMESTATE.may_load(deps.storage)?.unwrap();
    let mut winner_info: (u32, Option<Addr>) = (
        game.random_number
            .unwrap()
            .abs_diff(game.participants[0].choosed_number),
        Some(game.participants[0].address.clone()),
    );

    for player in &game.participants {
        let diff = game.random_number.unwrap().abs_diff(player.choosed_number);
        println!("chosed number: {}, diff: {}", player.choosed_number, diff);

        if diff < winner_info.0 {
            winner_info = (diff, Some(player.address.clone()))
        }
    }

    let finished_game: GameState = GameState {
        participants: game.participants,
        random_number: game.random_number,
        winner: winner_info.1,
        num_diff: winner_info.0,
        status: Some(Status::Finished {}),
    };

    GAMESTATE.save(deps.storage, &finished_game)?;

    Ok(Response::default())
}

pub fn try_reset_game(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let admin = ADMIN.load(deps.storage)?;

    if info.sender != admin.owner {
        return Err(ContractError::Unauthorized {});
    }

    let mut rng = rand::thread_rng();
    let number = rng.gen_range(0..=100);

    let game = GameState {
        participants: Vec::new(),
        random_number: Some(number),
        winner: None,
        num_diff: 0,
        status: Some(Status::Started {}),
    };

    GAMESTATE.save(deps.storage, &game)?;

    Ok(Response::new()
        .add_attribute("method", "restartGame")
        .add_attribute("sender", info.sender))
}

pub fn try_choose_number(
    deps: DepsMut,
    info: MessageInfo,
    choosed_number: u8,
) -> Result<Response, ContractError> {
    let game = GAMESTATE.load(deps.storage)?;

    if game.status == Some(Status::Finished {}) {
        return Err(ContractError::FinishedGame {});
    }

    let check_number_range = match choosed_number {
        0..=100 => true,
        _ => false,
    };

    if check_number_range == false {
        return Err(ContractError::NumberOutOfRange {});
    }

    let player = Player {
        address: info.sender.clone(),
        choosed_number: choosed_number.into(),
    };

    if game.participants.iter().any(|v| v == &player) {
        return Err(ContractError::PlayerAlreadyInGame {});
    }

    GAMESTATE.update(deps.storage, |mut game_state| -> Result<_, ContractError> {
        game_state.participants.push(player);
        Ok(game_state)
    })?;

    Ok(Response::new()
        .add_attribute("method", "chooseNumber")
        .add_attribute("sender", info.sender))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetGameInfo {} => to_binary(&query_game_info(deps)?),
        QueryMsg::GetAdminInfo {} => to_binary(&query_admin(deps)?),
    }
}

pub fn query_game_info(deps: Deps) -> StdResult<ResultGameInfo> {
    let game = GAMESTATE.may_load(deps.storage)?.unwrap();
    Ok(ResultGameInfo { result: game })
}

pub fn query_admin(deps: Deps) -> StdResult<ResultAdmin> {
    let admin = ADMIN.load(deps.storage)?;
    Ok(ResultAdmin { result: admin })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coins, from_binary};

    #[test]
    fn proper_initialization() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {
            name: String::from("Test"),
        };
        let info = mock_info("creator", &coins(1000, "earth"));
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        assert_eq!(0, res.messages.len());

        let info = mock_info("creator", &coins(1000, "earth"));
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAdminInfo {}).unwrap();
        let value: ResultAdmin = from_binary(&res).unwrap();
        assert_eq!(info.sender, value.result.owner);
    }

    #[test]
    fn check_admin() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            name: String::from("Test"),
        };

        let info = mock_info("creator", &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetAdminInfo {}).unwrap();
        let value: ResultAdmin = from_binary(&res).unwrap();
        assert_eq!(info.sender, value.result.owner);
    }

    #[test]
    fn check_init_game() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            name: String::from("Test"),
        };

        let info = mock_info("creator", &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGameInfo {}).unwrap();
        let value: ResultGameInfo = from_binary(&res).unwrap();
        let init_game = GameState {
            participants: Vec::new(),
            random_number: Some(0),
            status: Some(Status::Finished {}),
            num_diff: 0,
            winner: None,
        };
        assert_eq!(init_game, value.result)
    }

    #[test]
    fn reset_game() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            name: String::from("Test"),
        };

        let info = mock_info("creator", &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::ResetGame {}).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGameInfo {}).unwrap();
        let value: ResultGameInfo = from_binary(&res).unwrap();

        println!("random number is: {}", value.result.random_number.unwrap());
        assert_eq!(value.result.status, Some(Status::Started {}));
        assert_ne!(value.result.random_number.unwrap(), 0)
    }

    #[test]
    fn test_game() {
        let mut deps = mock_dependencies(&[]);
        let msg = InstantiateMsg {
            name: String::from("Test"),
        };

        let mut rng = rand::thread_rng();
        let mut random_numbers = Vec::new();
        let mut random_strings = Vec::new();

        for _ in 0..4 {
            random_numbers.push(rng.gen_range(0..=100));
            random_strings.push(rng.gen_range(0..=100).to_string());
        }

        let info = mock_info("creator", &coins(1000, "earth"));
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let _res = execute(deps.as_mut(), mock_env(), info, ExecuteMsg::ResetGame {}).unwrap();

        let msg = ExecuteMsg::ChooseNumber { number: 1 };
        let info = mock_info("participant_one", &coins(1234, "easz"));
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGameInfo {}).unwrap();
        let value: ResultGameInfo = from_binary(&res).unwrap();
        assert_eq!(1, value.result.participants.len());

        let msg = ExecuteMsg::ChooseNumber { number: 122 };
        let info = mock_info("invalid_participant", &coins(4321, "asd"));
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::NumberOutOfRange {}) => {}
            _ => panic!("expected error"),
        }

        for _ in 0..4 {
            let msg = ExecuteMsg::ChooseNumber {
                number: random_numbers.pop().unwrap(),
            };
            let info = mock_info(&random_strings.pop().unwrap(), &coins(1234, "easz"));
            let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        }

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGameInfo {}).unwrap();
        let value: ResultGameInfo = from_binary(&res).unwrap();
        // one has been added at start of this function
        assert_eq!(5, value.result.participants.len());

        let info = mock_info("notCreator", &coins(1000, "earth"));
        let msg = ExecuteMsg::EndGame {};
        let res = execute(deps.as_mut(), mock_env(), info, msg);

        match res {
            Err(ContractError::Unauthorized {}) => {}
            _ => panic!("expected error"),
        }

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = ExecuteMsg::EndGame {};
        let _res = execute(deps.as_mut(), mock_env(), info, msg);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::GetGameInfo {}).unwrap();
        let value: ResultGameInfo = from_binary(&res).unwrap();
        // one has been added at start of this function
        println!("{:?}", value.result);
        // assert_eq!(value.result.status, Some(Status::Finished {}))
    }
}

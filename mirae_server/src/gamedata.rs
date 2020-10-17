use crate::deser::gamemode::GameData;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use std::sync::{RwLock, RwLockReadGuard};

lazy_static! {
    static ref GAMEDATA: RwLock<Result<GameData>> =
        { RwLock::new(Err(anyhow!("gamedata is unassigned!"))) };
}

pub fn assign(gamedata: GameData) -> Result<()> {
    if let Ok(r) = GAMEDATA.try_read() {
        if (*r).is_err() {
            if let Ok(mut w) = (GAMEDATA).try_write() {
                *w = Ok(gamedata);
                Ok(())
            } else {
                Err(anyhow!("could not write data!"))
            }
        } else {
            Err(anyhow!(
                "gamedata has already been assigned, and cannot be reassigned!"
            ))
        }
    } else {
        Err(anyhow!("Cannot access gamedata!"))
    }
}

pub fn get() -> Result<RwLockReadGuard<'static, Result<GameData>>> {
    if let Ok(res) = GAMEDATA.read() {
        return Ok(res);
    } else {
        return Err(anyhow!(
            "RwLock is poisoned! This should never happen because the data is only assigned once!"
        ));
    }
}

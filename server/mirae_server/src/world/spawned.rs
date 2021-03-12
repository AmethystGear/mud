use crate::location::Vector2;
use anyhow::{anyhow, Result};
use std::{sync::atomic::{Ordering, AtomicU64}, collections::HashMap};

#[derive(Debug, Hash, Eq, PartialEq, Clone, Copy)]
pub struct Handle(u64);

static HANDLE: AtomicU64 = AtomicU64::new(0);

pub trait Creature : Located {

}

pub trait Located {
    fn loc(&self) -> Vector2;
    fn set_loc(&mut self, loc: Vector2);
}

#[derive(Debug)]
pub struct Spawned<T>
where
    T: Creature,
{
    handle_to_object: HashMap<Handle, T>,
    posn_to_handle: HashMap<Vector2, Handle>,
    handle_to_posn: HashMap<Handle, Vector2>,
}

impl<T> Spawned<T>
where
    T: Creature,
{
    pub fn new() -> Self {
        Spawned {
            handle_to_object: HashMap::new(),
            posn_to_handle: HashMap::new(),
            handle_to_posn: HashMap::new()
        }
    }

    pub fn mov(&mut self, start: Vector2, end: Vector2) -> Result<()> {
        if let Some(_) = self.posn_to_handle.get(&end) {
            Err(anyhow!(
                "cannot move from {:?} to {:?} because there is already an object at {:?}",
                start,
                end,
                end
            ))
        } else {
            if let Ok(object_handle) = self.get_handle_by_loc(start) {
                self.handle_to_posn.remove(&object_handle);
                self.posn_to_handle.remove(&start);
                self.handle_to_posn.insert(object_handle, end);
                self.posn_to_handle.insert(end, object_handle);
                self.handle_to_object
                    .get_mut(&object_handle)
                    .ok_or_else(|| anyhow!("invariant failure"))?
                    .set_loc(end);
                Ok(())
            } else {
                Err(anyhow!(
                    "cannot move from {:?} to {:?} because there is no object at {:?}",
                    start,
                    end,
                    start
                ))
            }
        }
    }

    pub fn del(&mut self, handle: Handle) -> Result<()> {
        if let Some(posn) = self.handle_to_posn.remove(&handle) {
            self.handle_to_object.remove(&handle);
            self.posn_to_handle.remove(&posn);
            Ok(())
        } else {
            Err(anyhow!("cannot delete object with handle {}", handle.0))
        }
    }

    pub fn spawn(&mut self, object: T) -> Result<Handle> {
        let loc = object.loc();
        if let Ok(_) = self.get_handle_by_loc(loc) {
            Err(anyhow!(
                "cannot create object at {:?} because there already is an object there",
                loc
            ))
        } else {
            let handle = Handle(HANDLE.fetch_add(1, Ordering::SeqCst));
            self.handle_to_object.insert(handle, object);
            self.posn_to_handle.insert(loc, handle);
            self.handle_to_posn.insert(handle, loc);
            Ok(handle)
        }
    }

    pub fn get_handle_by_loc(&self, loc: Vector2) -> Result<Handle> {
        Ok(self
            .posn_to_handle
            .get(&loc)
            .ok_or_else(|| anyhow!("no object at the location {:?}", loc))?
            .clone())
    }

    pub fn get_object_by_handle(&self, handle: Handle) -> Result<&T> {
        Ok(self
            .handle_to_object
            .get(&handle)
            .ok_or_else(|| anyhow!("no object exists with handle {}", handle.0))?)
    }

    pub fn get_object_by_handle_mut(&mut self, handle: Handle) -> Result<&mut T> {
        Ok(self
            .handle_to_object
            .get_mut(&handle)
            .ok_or_else(|| anyhow!("no object exists with handle {}", handle.0))?)
    }
}

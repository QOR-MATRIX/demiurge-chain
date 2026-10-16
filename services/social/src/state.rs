//! What every request reaches.

use crate::hub::Hub;
use crate::identity::Identity;
use crate::store::Store;

pub struct AppState {
    pub store: Store,
    pub identity: Identity,
    pub hub: Hub,
}

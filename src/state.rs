use crate::models::{FeatureFlag, Override};
use dashmap::DashMap;
use std::collections::HashMap;

pub struct AppState {
    pub flags: DashMap<String, FeatureFlag>,
    pub overrides: DashMap<String, HashMap<String, Override>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            flags: DashMap::new(),
            overrides: DashMap::new(),
        }
    }
}

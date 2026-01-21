pub struct QuadContext{
    pub state: Arc<RwLock<QuadState>>,
    pub commands: Arc<Mutex<Vec<QuadCommand>>>,
}
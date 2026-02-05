use std::sync::Arc;

use tokio::sync::{RwLock, mpsc};

/// A factory function that spawns an actor and returns its message sender.
pub type ActorSpawner<T> = Box<dyn Fn() -> mpsc::Sender<T> + Send + Sync>;

/// A wrapper around `mpsc::Sender` that automatically detects channel closure
/// and respawns the actor using the provided factory.
pub struct Sender<T> {
    /// The current active channel sender.
    tx: Arc<RwLock<mpsc::Sender<T>>>,
    /// The function to call when we need to respawn the actor.
    factory: Arc<ActorSpawner<T>>,
    /// A name for logging purposes.
    actor_name: &'static str,
}

impl<T> Sender<T>
where
    T: Send + 'static,
{
    /// Creates a new Sender.
    ///
    /// This immediately calls `factory` once to start the initial actor.
    pub fn new(
        actor_name: &'static str,
        factory: impl Fn() -> mpsc::Sender<T> + Send + Sync + 'static,
    ) -> Self {
        let factory = Box::new(factory);
        let tx = factory();

        Self {
            tx: Arc::new(RwLock::new(tx)),
            factory: Arc::new(factory),
            actor_name,
        }
    }

    /// Sends a message. If the actor is dead, it respawns it and retries once.
    #[allow(clippy::significant_drop_tightening)]
    pub async fn send(&self, msg: T) -> Result<(), mpsc::error::SendError<T>> {
        let tx = self.tx.read().await;
        if let Err(mpsc::error::SendError(msg_back)) = tx.send(msg).await {
            drop(tx);

            let mut tx_guard = self.tx.write().await;

            if tx_guard.is_closed() {
                warn!("{} disconnected. Respawning instance...", self.actor_name);

                let new_tx = (self.factory)();
                *tx_guard = new_tx;

                info!("{} respawned successfully.", self.actor_name);
            }

            return tx_guard.send(msg_back).await;
        }

        Ok(())
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            factory: self.factory.clone(),
            actor_name: self.actor_name,
        }
    }
}

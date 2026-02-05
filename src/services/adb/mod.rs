use std::net::SocketAddrV4;
use std::sync::Arc;

pub use error::AdbError;
pub use handle::AdbHandle;
use scrcpy_launcher::adb::server::{ADBServer, DeviceLong};
use scrcpy_launcher::adb::server_device::ADBServerDevice;
use tokio::sync::{mpsc, oneshot};
use tokio::task::spawn_blocking;

use crate::services::adb::utils::ensure_connection;

mod error;
mod handle;
mod utils;

pub type DeviceId = String;

pub type AdbResult<T = ()> = Result<T, AdbError>;

/// Commands that can be sent to the [`AdbActor`].
#[derive(Debug)]
pub enum AdbCommand {
    /// Request a list of currently connected ADB devices.
    GetDevices {
        respond_to: oneshot::Sender<AdbResult<Vec<DeviceLong>>>,
    },
    /// Request a server device by its identifier.
    GetDevice {
        identifier: DeviceId,
        respond_to: oneshot::Sender<AdbResult<ADBServerDevice>>,
    },
    /// Connect a device over tcp via its IPv4 socket address.
    ConnectDevice {
        address: SocketAddrV4,
        respond_to: oneshot::Sender<AdbResult>,
    },
    /// Disconnect a device over tcp via its IPv4 socket address.
    DisconnectDevice {
        address: SocketAddrV4,
        respond_to: oneshot::Sender<AdbResult>,
    },
    /// Signals the service to exit.
    Exit,
}

/// Actor managing the ADB server lifecycle and operations.
struct AdbActor {
    /// Shared ADB server instance.
    adb: Arc<parking_lot::Mutex<Option<ADBServer>>>,
    /// The receiver for incoming commands.
    rx: mpsc::Receiver<AdbCommand>,
}

impl AdbActor {
    /// Creates a new [`AdbActor`] instance.
    ///
    /// # Arguments
    ///
    /// - `receiver` - Channel receiver for processing [`AdbCommand`] messages.
    fn new(rx: mpsc::Receiver<AdbCommand>) -> Self {
        Self {
            adb: Arc::default(),
            rx,
        }
    }

    /// Runs the actor's main event loop.
    ///
    /// Processes messages until the channel is closed or a [`AdbCommand::Exit`] message is received.
    async fn run(mut self) {
        debug!("Event loop started.");

        while let Some(msg) = self.rx.recv().await {
            match msg {
                AdbCommand::GetDevices { respond_to } => {
                    self.handle_get_devices(respond_to).await;
                }
                AdbCommand::GetDevice {
                    identifier,
                    respond_to,
                } => {
                    self.handle_get_device(identifier, respond_to).await;
                }
                AdbCommand::ConnectDevice {
                    address,
                    respond_to,
                } => {
                    self.handle_connect_device(address, respond_to).await;
                }
                AdbCommand::DisconnectDevice {
                    address,
                    respond_to,
                } => {
                    self.handle_disconnect_device(address, respond_to).await;
                }
                AdbCommand::Exit => {
                    debug!("Shutting down, {} messages remaining.", self.rx.len());
                    self.rx.close();
                }
            }
        }

        // self.exit();
    }

    // /// Performs a clean exit sequence.
    // fn exit(self) {
    //     // NOTE: Do not kill the ADB server, it might disrupt other processes,
    //     // and takes 5 seconds to start again
    //
    //     // let mut adb_guard = self.adb.lock();
    //     //
    //     // if let Some(adb) = adb_guard.as_mut() {
    //     //     debug!("Killing ADB server process.");
    //     //     if let Err(err) = adb.kill() {
    //     //         warn!("Failed to kill ADB server: {err}");
    //     //     }
    //     // } else {
    //     //     debug!("No ADB server instance to kill.");
    //     // }
    // }
}

impl AdbActor {
    /// Request a list of currently connected ADB devices.
    async fn handle_get_devices(
        &self,
        respond_to: oneshot::Sender<Result<Vec<DeviceLong>, AdbError>>,
    ) {
        let adb_clone = self.adb.clone();

        #[allow(clippy::significant_drop_tightening)]
        let result = spawn_blocking(move || {
            let mut adb_lock = adb_clone.lock();

            let adb = ensure_connection(&mut adb_lock)?;

            adb.devices_long().map_err(Into::into)
        })
        .await
        .map_err(Into::into);

        if respond_to.send(result.flatten()).is_err() {
            warn!("Failed to send response: Receiver dropped.");
        }
    }

    /// Request a server device by its identifier.
    async fn handle_get_device(
        &self,
        identifier: DeviceId,
        respond_to: oneshot::Sender<Result<ADBServerDevice, AdbError>>,
    ) {
        let adb_clone = self.adb.clone();

        #[allow(clippy::significant_drop_tightening)]
        let result = spawn_blocking(move || {
            let mut adb_lock = adb_clone.lock();

            let adb = ensure_connection(&mut adb_lock)?;
            adb.get_device_by_name(&identifier).map_err(Into::into)
        })
        .await
        .map_err(Into::into);

        if respond_to.send(result.flatten()).is_err() {
            warn!("Failed to send response: Receiver dropped.");
        }
    }

    /// Connect a device over tcp via its IPv4 socket address.
    async fn handle_connect_device(
        &self,
        address: SocketAddrV4,
        respond_to: oneshot::Sender<Result<(), AdbError>>,
    ) {
        let adb_clone = self.adb.clone();

        #[allow(clippy::significant_drop_tightening)]
        let result = spawn_blocking(move || {
            let mut adb_lock = adb_clone.lock();

            let adb = ensure_connection(&mut adb_lock)?;
            adb.connect_device(address).map_err(Into::into)
        })
        .await
        .map_err(Into::into);

        if respond_to.send(result.flatten()).is_err() {
            warn!("Failed to send response: Receiver dropped.");
        }
    }

    /// Disconnect a device over tcp via its IPv4 socket address.
    async fn handle_disconnect_device(
        &self,
        address: SocketAddrV4,
        respond_to: oneshot::Sender<Result<(), AdbError>>,
    ) {
        let adb_clone = self.adb.clone();

        #[allow(clippy::significant_drop_tightening)]
        let result = spawn_blocking(move || {
            let mut adb_lock = adb_clone.lock();

            let adb = ensure_connection(&mut adb_lock)?;
            adb.disconnect_device(address).map_err(Into::into)
        })
        .await
        .map_err(Into::into);

        if respond_to.send(result.flatten()).is_err() {
            warn!("Failed to send response: Receiver dropped.");
        }
    }
}

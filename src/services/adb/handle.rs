use std::net::SocketAddrV4;

use scrcpy_launcher::adb::server::DeviceLong;
use scrcpy_launcher::adb::server_device::ADBServerDevice;
use tokio::sync::{mpsc, oneshot};

use crate::services::adb::{AdbActor, AdbCommand, AdbError, AdbResult, DeviceId};
use crate::services::utils::sender::Sender;

/// A thread-safe handle for interacting with the ADB service.
#[derive(Clone)]
pub struct AdbHandle {
    sender: Sender<AdbCommand>,
}

impl AdbHandle {
    const BUFFER: usize = 32;

    /// Spawns the ADB actor and returns a communication handle.
    pub fn new() -> Self {
        Self {
            sender: Sender::new("AdbActor", move || {
                let (tx, rx) = mpsc::channel(Self::BUFFER);
                tokio::spawn(AdbActor::new(rx).run());
                tx
            }),
        }
    }

    /// Requests a list of connected devices from the ADB server.
    ///
    /// # Errors
    ///
    /// Returns [`AdbError::Disconnected`] if the actor task has terminated
    /// or if the response channel is dropped unexpectedly.
    pub async fn get_devices(&self) -> AdbResult<Vec<DeviceLong>> {
        let (tx, rx) = oneshot::channel();
        let cmd = AdbCommand::GetDevices { respond_to: tx };

        self.sender
            .send(cmd)
            .await
            .map_err(|_| AdbError::ChannelClosed)?;

        rx.await.map_err(|_| AdbError::ChannelClosed)?
    }

    /// Request a server device by its identifier.
    ///
    /// # Errors
    ///
    /// Returns [`AdbError::Disconnected`] if the actor task has terminated
    /// or if the response channel is dropped unexpectedly.
    pub async fn get_device(&self, identifier: DeviceId) -> AdbResult<ADBServerDevice> {
        let (tx, rx) = oneshot::channel();
        let cmd = AdbCommand::GetDevice {
            identifier,
            respond_to: tx,
        };

        self.sender
            .send(cmd)
            .await
            .map_err(|_| AdbError::ChannelClosed)?;

        rx.await.map_err(|_| AdbError::ChannelClosed)?
    }

    /// Connect a device over tcp via its IPv4 socket address.
    ///
    /// # Errors
    ///
    /// Returns [`AdbError::Disconnected`] if the actor task has terminated
    /// or if the response channel is dropped unexpectedly.
    pub async fn connect_device(&self, address: SocketAddrV4) -> AdbResult {
        let (tx, rx) = oneshot::channel();
        let cmd = AdbCommand::ConnectDevice {
            address,
            respond_to: tx,
        };

        self.sender
            .send(cmd)
            .await
            .map_err(|_| AdbError::ChannelClosed)?;

        rx.await.map_err(|_| AdbError::ChannelClosed)?
    }

    /// Disconnect a device over tcp via its IPv4 socket address.
    ///
    /// # Errors
    ///
    /// Returns [`AdbError::Disconnected`] if the actor task has terminated
    /// or if the response channel is dropped unexpectedly.
    pub async fn disconnect_device(&self, address: SocketAddrV4) -> AdbResult {
        let (tx, rx) = oneshot::channel();
        let cmd = AdbCommand::DisconnectDevice {
            address,
            respond_to: tx,
        };

        self.sender
            .send(cmd)
            .await
            .map_err(|_| AdbError::ChannelClosed)?;

        rx.await.map_err(|_| AdbError::ChannelClosed)?
    }

    /// Signals the ADB service to exit.
    pub async fn exit(&self) {
        let _ = self.sender.send(AdbCommand::Exit).await;
    }
}

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddrV4};

use scrcpy_launcher::adb::server::ADBServer;

use crate::ADB_PATH;
use crate::services::adb::AdbError;

/// Ensures a valid connection to the ADB server, starting it if necessary.
///
/// # Blocking
///
/// Do not call from async context.
pub fn ensure_connection(adb_lock: &mut Option<ADBServer>) -> Result<&mut ADBServer, AdbError> {
    let healthy = adb_lock.as_mut().is_some_and(|s| s.server_status().is_ok());
    if healthy {
        return Ok(adb_lock.as_mut().expect("Adb server is Some()"));
    }

    let mut adb = create_adb_server(Some(ADB_PATH.to_string()));
    if adb.server_status().is_err() {
        let _ = adb.kill();
        ADBServer::start(&HashMap::default(), &Some(ADB_PATH.to_string()));

        adb.server_status()?;
    }

    Ok(adb_lock.insert(adb))
}

/// Creates a new ADB server
///
/// # Blocking
///
/// Do not call from async context.
fn create_adb_server(adb_path: Option<String>) -> ADBServer {
    const SERVER_IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
    const SERVER_PORT: u16 = 5037;

    ADBServer::new_from_path(SocketAddrV4::new(SERVER_IP, SERVER_PORT), adb_path)
}

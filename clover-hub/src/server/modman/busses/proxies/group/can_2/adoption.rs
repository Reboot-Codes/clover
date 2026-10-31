use std::time::Duration;

use anyhow::anyhow;
use can_isotp_interface::{
  IsoTpAsyncEndpoint,
  RecvControl,
  RecvStatus,
};
use linux_socketcan_iso_tp::TokioSocketCanIsoTp;
use tracing::{
  debug,
  info,
};

use carbon_fiber::{
  adoption::{
    hub::HubAdoptionContext,
    models::{
      AdoptionMessage,
      AdoptionState,
    },
  },
  models::BusMessage,
};

/// Runs the sequential CarbonFiber adoption handshake over open CAN ISO-TP sockets.
///
/// Sequence:
/// 1. Hub sends `ProbeMessage { version, id }`
/// 2. Module replies with `HelloMessage { version, rfqdn, salt }`
/// 3. Hub sends `KeyHOMIMessage { hub_pub }`
/// 4. Module replies with `KeyHIMOMessage { module_pub, hub_conf }`
/// 5. Hub sends `ConfirmMessage { module_conf }`
///
/// Both sides transition to `AdoptionState::Confirmed` upon completion.
pub async fn run_can2_adoption_handshake(
  rx_socket: &mut TokioSocketCanIsoTp,
  tx_socket: &mut TokioSocketCanIsoTp,
  module_id: &str,
  expected_rfqdn: &str,
  pin: [u8; 6],
  timeout: Duration,
) -> Result<(), anyhow::Error> {
  let mut adoption = HubAdoptionContext::new(module_id, expected_rfqdn, pin);

  debug!("Starting CAN 2 adoption handshake for module: {module_id} (expected RFQDN: '{expected_rfqdn}')...");

  // Step 1: Send Probe
  let probe = adoption.gen_probe();
  let probe_msg = BusMessage::Adoption(AdoptionMessage::Probe(probe));
  let probe_bytes = postcard::to_allocvec(&probe_msg)
    .map_err(|e| anyhow!("Failed to serialize ProbeMessage: {e}"))?;

  tx_socket
    .send_to(0, &probe_bytes, timeout)
    .await
    .map_err(|e| anyhow!("Failed to send ProbeMessage to module {module_id}: {e:?}"))?;

  // Step 2: Receive Hello
  let mut hello_bytes = Vec::new();
  let recv_res = rx_socket
    .recv_one(timeout, |_meta, data| {
      hello_bytes.extend_from_slice(data);
      Ok(RecvControl::Continue)
    })
    .await
    .map_err(|e| anyhow!("Error receiving HelloMessage from {module_id}: {e:?}"))?;

  if recv_res == RecvStatus::TimedOut || hello_bytes.is_empty() {
    return Err(anyhow!(
      "Timed out waiting for HelloMessage from module {module_id}"
    ));
  }

  let hello_msg = postcard::from_bytes::<BusMessage>(&hello_bytes)
    .map_err(|e| anyhow!("Failed to decode HelloMessage from module {module_id}: {e}"))?;

  match hello_msg {
    BusMessage::Adoption(AdoptionMessage::Hello(hello)) => {
      if !expected_rfqdn.is_empty() && hello.rfqdn != expected_rfqdn {
        return Err(anyhow!(
          "Module RFQDN mismatch for {module_id}: expected '{expected_rfqdn}', got '{}'",
          hello.rfqdn
        ));
      }
      adoption
        .parse_hello(&hello)
        .map_err(|reason| anyhow!("Module {module_id} rejected during Hello: {reason:?}"))?;
      info!(
        "Module {module_id} responded to Probe with Hello (RFQDN: {}, version: {:?})",
        hello.rfqdn, hello.version
      );
    }
    BusMessage::Adoption(AdoptionMessage::Abortion(abort)) => {
      return Err(anyhow!(
        "Module {module_id} aborted adoption during Probe: {:?}",
        abort.reason
      ));
    }
    other => {
      return Err(anyhow!(
        "Unexpected message waiting for Hello from {module_id}: {other:?}"
      ));
    }
  }

  // Step 3: Send KeyHOMI
  let homi = adoption.gen_key_homi();
  let homi_msg = BusMessage::Adoption(AdoptionMessage::KeyHOMI(homi));
  let homi_bytes = postcard::to_allocvec(&homi_msg)
    .map_err(|e| anyhow!("Failed to serialize KeyHOMIMessage: {e}"))?;

  tx_socket
    .send_to(0, &homi_bytes, timeout)
    .await
    .map_err(|e| anyhow!("Failed to send KeyHOMIMessage to {module_id}: {e:?}"))?;

  // Step 4: Receive KeyHIMO
  let mut himo_bytes = Vec::new();
  let recv_res = rx_socket
    .recv_one(timeout, |_meta, data| {
      himo_bytes.extend_from_slice(data);
      Ok(RecvControl::Continue)
    })
    .await
    .map_err(|e| anyhow!("Error receiving KeyHIMOMessage from {module_id}: {e:?}"))?;

  if recv_res == RecvStatus::TimedOut || himo_bytes.is_empty() {
    return Err(anyhow!(
      "Timed out waiting for KeyHIMOMessage from module {module_id}"
    ));
  }

  let himo_msg = postcard::from_bytes::<BusMessage>(&himo_bytes)
    .map_err(|e| anyhow!("Failed to decode KeyHIMOMessage from module {module_id}: {e}"))?;

  match himo_msg {
    BusMessage::Adoption(AdoptionMessage::KeyHIMO(himo)) => {
      adoption
        .parse_key_himo(&himo)
        .map_err(|reason| anyhow!("Key exchange failed with module {module_id}: {reason:?}"))?;
      debug!("Key exchange verified for module {module_id}");
    }
    BusMessage::Adoption(AdoptionMessage::Abortion(abort)) => {
      return Err(anyhow!(
        "Module {module_id} aborted during key exchange: {:?}",
        abort.reason
      ));
    }
    other => {
      return Err(anyhow!(
        "Unexpected message waiting for KeyHIMO from {module_id}: {other:?}"
      ));
    }
  }

  // Step 5: Send Confirm
  let confirm = adoption.gen_confirm();
  let confirm_msg = BusMessage::Adoption(AdoptionMessage::Confirm(confirm));
  let confirm_bytes = postcard::to_allocvec(&confirm_msg)
    .map_err(|e| anyhow!("Failed to serialize ConfirmMessage: {e}"))?;

  tx_socket
    .send_to(0, &confirm_bytes, timeout)
    .await
    .map_err(|e| anyhow!("Failed to send ConfirmMessage to {module_id}: {e:?}"))?;

  if adoption.state != AdoptionState::Confirmed {
    return Err(anyhow!(
      "Adoption state did not reach Confirmed for module {module_id}"
    ));
  }

  info!("Successfully completed CarbonFiber adoption handshake for module: {module_id}!");
  Ok(())
}

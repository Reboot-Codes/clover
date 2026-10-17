//! # Module Adoption Workflow
//!
//! For convinience, all adoption related messages are housed within the [models::AdoptionMessage] enum.
//!
//! Hub specific implementations are in [hub], while module specific implementations are located in [module]; with shared types being found in [models].
//!
//! The adoption process is as follows:
//!
//! ```mermaid
//! sequenceDiagram
//!    autonumber
//!    actor User
//!    participant Hub as Hub (Host)
//!    participant Module as Module (Node)
//!
//!    Note over Hub,Module: Discovery & Compatibility Check
//!    Hub->>Module: Probe(version, id)
//!    alt Version mismatch / unsupported ID
//!        Module-->>Hub: Abortion(Incompatible)
//!    else Version compatible
//!        Module->>Hub: Hello(version, rfqdn, salt)
//!    end
//!
//!    Note over User,Module: Out-of-Band PIN Display / Discovery
//!    Module-->>User: Displays or exposes pairing PIN
//!
//!    Note over Hub,Module: PAKE Key Exchange
//!    User->>Hub: Inputs pairing PIN
//!    Hub->>Hub: Compute ephemeral keypair & PAKE share
//!    Hub->>Module: KeyHOMI(hub_pub)
//!
//!    Module->>Module: Compute shared secret & confirmation tag
//!    alt Module math error or bad state
//!        Module-->>Hub: Abortion(KeyHIMOFail)
//!    else Share valid
//!        Module->>Hub: KeyHIMO(module_pub, hub_conf)
//!    end
//!
//!    Note over Hub,Module: Mutual Authentication & Confirmation
//!    Hub->>Hub: Verify hub_conf & compute module_conf
//!    alt Wrong PIN or verification mismatch
//!        Hub-->>Module: Abortion(ConfirmHOMIFail)
//!    else Confirmation valid
//!        Hub->>Module: Confirm(module_conf)
//!        Module->>Module: Verify module_conf
//!        alt Module verification fails
//!            Module-->>Hub: Abortion(ConfirmHIMOFail)
//!        else Module verification succeeds
//!            Note over Hub,Module: Adoption Complete (Secure Session Established)
//!        end
//!    end
//!
//!    opt User manually cancels anytime
//!        Hub-->>Module: Abortion(UserCancel)
//!    end
//! ```
//!

pub mod hub;
pub mod models;
pub mod module;

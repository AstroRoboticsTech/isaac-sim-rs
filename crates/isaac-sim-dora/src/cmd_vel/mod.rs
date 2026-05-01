// SPDX-License-Identifier: MPL-2.0
//! cmd_vel: bidirectional articulation channel.
//!
//! [`subscribe`] is the dora→bridge direction: decode a Twist Arrow
//! batch from a dora input and republish it into the bridge's cmd_vel
//! producer slot. The C++ apply node then polls that slot.
//!
//! [`publish`] is the bridge→dora direction: hook into the producer
//! registry's observer channel and emit each Twist as a dora output.
//! Lets dora dataflows log, replay, or fan out the actuation stream
//! without coupling to whichever Rust source originally published it.

pub mod publish;
pub(crate) mod subscribe;

pub(crate) use subscribe::start_cmd_vel_subscriber;

//! End-to-end integration test for the Rust→C++ cmd_vel path.
//!
//! Build a Twist Arrow batch the way an upstream dora node would,
//! drive it through the dora subscriber's decode path, and assert
//! `peek_cmd_vel` reads back the exact value the C++ apply node would
//! see via `poll_cmd_vel`.
//!
//! Mirrors the fixture-replay pattern used for sensor publishers,
//! adapted for the reverse data direction: there's no captured
//! annotator output to consume, so the test produces the Arrow batch
//! itself and asserts the decode + producer-slot publish chain
//! preserves every Twist component.

use std::sync::Arc;

use arrow::array::{ArrayRef, StructArray};
use isaac_sim_arrow::cmd_vel::{from_struct_array, to_record_batch, CmdVel as ArrowCmdVel};
use isaac_sim_bridge::{peek_cmd_vel, register_cmd_vel_producer, CmdVel as BridgeCmdVel};

fn build_struct(twist: &ArrowCmdVel) -> ArrayRef {
    let batch = to_record_batch(twist).expect("convert");
    Arc::new(StructArray::from(batch))
}

fn republish(target: &str, array: &ArrayRef) {
    let array = array
        .as_any()
        .downcast_ref::<StructArray>()
        .expect("StructArray");
    let twist = from_struct_array(array).expect("decode");
    let slot = register_cmd_vel_producer(target);
    slot.publish(BridgeCmdVel {
        linear_x: twist.linear_x,
        linear_y: twist.linear_y,
        linear_z: twist.linear_z,
        angular_x: twist.angular_x,
        angular_y: twist.angular_y,
        angular_z: twist.angular_z,
        timestamp_ns: twist.timestamp_ns,
    });
}

#[test]
fn arrow_to_slot_preserves_every_field() {
    let target = "/integ/cmd_vel/preserves_every_field";
    let twist = ArrowCmdVel {
        linear_x: 0.4,
        linear_y: 0.05,
        linear_z: -0.01,
        angular_x: 0.02,
        angular_y: -0.03,
        angular_z: 0.3,
        timestamp_ns: 1_700_000_000,
    };
    let array = build_struct(&twist);
    republish(target, &array);

    let polled = peek_cmd_vel(target).expect("slot has value");
    assert!((polled.linear_x - 0.4).abs() < 1e-6);
    assert!((polled.linear_y - 0.05).abs() < 1e-6);
    assert!((polled.linear_z + 0.01).abs() < 1e-6);
    assert!((polled.angular_x - 0.02).abs() < 1e-6);
    assert!((polled.angular_y + 0.03).abs() < 1e-6);
    assert!((polled.angular_z - 0.3).abs() < 1e-6);
    assert_eq!(polled.timestamp_ns, 1_700_000_000);
}

#[test]
fn separate_targets_do_not_alias() {
    let target_a = "/integ/cmd_vel/no_alias_a";
    let target_b = "/integ/cmd_vel/no_alias_b";

    let twist_a = ArrowCmdVel {
        linear_x: 1.5,
        timestamp_ns: 1,
        ..ArrowCmdVel::default()
    };
    let twist_b = ArrowCmdVel {
        linear_x: -2.5,
        timestamp_ns: 2,
        ..ArrowCmdVel::default()
    };

    republish(target_a, &build_struct(&twist_a));
    republish(target_b, &build_struct(&twist_b));

    assert!((peek_cmd_vel(target_a).expect("a").linear_x - 1.5).abs() < 1e-6);
    assert!((peek_cmd_vel(target_b).expect("b").linear_x + 2.5).abs() < 1e-6);
}

#[test]
fn last_publish_wins_per_target() {
    let target = "/integ/cmd_vel/last_publish_wins";
    let earlier = ArrowCmdVel {
        linear_x: 0.1,
        timestamp_ns: 100,
        ..ArrowCmdVel::default()
    };
    let later = ArrowCmdVel {
        linear_x: 0.9,
        timestamp_ns: 200,
        ..ArrowCmdVel::default()
    };
    republish(target, &build_struct(&earlier));
    republish(target, &build_struct(&later));

    let polled = peek_cmd_vel(target).expect("slot has value");
    assert!((polled.linear_x - 0.9).abs() < 1e-6);
    assert_eq!(polled.timestamp_ns, 200);
}

#[test]
fn observer_sees_every_publish() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    use isaac_sim_bridge::register_cmd_vel_consumer;

    let target = "/integ/cmd_vel/observer_sees_publish";
    let count = Arc::new(AtomicUsize::new(0));
    let last = Arc::new(Mutex::new(None::<f32>));

    let count_c = Arc::clone(&count);
    let last_c = Arc::clone(&last);
    register_cmd_vel_consumer(move |seen_target, twist| {
        if seen_target != target {
            return;
        }
        count_c.fetch_add(1, Ordering::SeqCst);
        *last_c.lock().unwrap() = Some(twist.linear_x);
    });

    republish(
        target,
        &build_struct(&ArrowCmdVel {
            linear_x: 0.25,
            timestamp_ns: 1,
            ..ArrowCmdVel::default()
        }),
    );
    republish(
        target,
        &build_struct(&ArrowCmdVel {
            linear_x: 0.75,
            timestamp_ns: 2,
            ..ArrowCmdVel::default()
        }),
    );

    assert_eq!(count.load(Ordering::SeqCst), 2);
    let seen = last.lock().unwrap().expect("observer saw at least one");
    assert!((seen - 0.75).abs() < 1e-6);
}

#[test]
fn dora_publisher_arrow_round_trips_through_observer() {
    use isaac_sim_arrow::cmd_vel::from_struct_array;
    use isaac_sim_dora::cmd_vel::publish::build_struct_array as build_pub_struct;

    let target = "/integ/cmd_vel/dora_publisher_round_trip";

    // Subscriber side: build Arrow from a synthetic Twist + drive it
    // into the producer slot exactly as the dora subscriber would.
    let original = ArrowCmdVel {
        linear_x: 0.31,
        linear_y: -0.07,
        linear_z: 0.0,
        angular_x: 0.0,
        angular_y: 0.0,
        angular_z: 1.21,
        timestamp_ns: 4242,
    };
    republish(target, &build_struct(&original));

    // Publisher side: rebuild Arrow from the value held in the slot
    // (proxy for what the observer-driven publisher would emit on its
    // dora output). Decoding that StructArray must produce the same
    // Twist we put in.
    let slot_value = peek_cmd_vel(target).expect("slot has value");
    let array = build_pub_struct(&slot_value).expect("rebuild");
    let decoded = from_struct_array(&array).expect("decode");

    assert!((decoded.linear_x - original.linear_x).abs() < 1e-6);
    assert!((decoded.linear_y - original.linear_y).abs() < 1e-6);
    assert!((decoded.angular_z - original.angular_z).abs() < 1e-6);
    assert_eq!(decoded.timestamp_ns, original.timestamp_ns);
}

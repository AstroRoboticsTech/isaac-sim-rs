// SPDX-License-Identifier: MPL-2.0
//! Arrow encoder and decoder for the chassis odometry channel.
use std::sync::{Arc, OnceLock};

use arrow::array::{Array, ArrayRef, Float64Array, Int64Array, StringArray, StructArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

/// Borrowed view of a single chassis odometry sample, used as input to [`to_record_batch`].
#[allow(missing_docs)]
pub struct Odometry<'a> {
    pub chassis_frame_id: &'a str,
    pub odom_frame_id: &'a str,
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub orientation_w: f64,
    pub orientation_x: f64,
    pub orientation_y: f64,
    pub orientation_z: f64,
    pub lin_vel_x: f64,
    pub lin_vel_y: f64,
    pub lin_vel_z: f64,
    pub ang_vel_x: f64,
    pub ang_vel_y: f64,
    pub ang_vel_z: f64,
    pub timestamp_ns: i64,
}

/// Owned variant returned by [`from_struct_array`].
#[derive(Debug, Clone, PartialEq)]
#[allow(missing_docs)]
pub struct OdometryOwned {
    pub chassis_frame_id: String,
    pub odom_frame_id: String,
    pub position_x: f64,
    pub position_y: f64,
    pub position_z: f64,
    pub orientation_w: f64,
    pub orientation_x: f64,
    pub orientation_y: f64,
    pub orientation_z: f64,
    pub lin_vel_x: f64,
    pub lin_vel_y: f64,
    pub lin_vel_z: f64,
    pub ang_vel_x: f64,
    pub ang_vel_y: f64,
    pub ang_vel_z: f64,
    pub timestamp_ns: i64,
}

/// Stable Arrow schema for an `Odometry` record batch.
pub fn schema() -> SchemaRef {
    static SCHEMA: OnceLock<SchemaRef> = OnceLock::new();
    SCHEMA
        .get_or_init(|| {
            Arc::new(Schema::new(vec![
                Field::new("chassis_frame_id", DataType::Utf8, false),
                Field::new("odom_frame_id", DataType::Utf8, false),
                Field::new("position_x", DataType::Float64, false),
                Field::new("position_y", DataType::Float64, false),
                Field::new("position_z", DataType::Float64, false),
                Field::new("orientation_w", DataType::Float64, false),
                Field::new("orientation_x", DataType::Float64, false),
                Field::new("orientation_y", DataType::Float64, false),
                Field::new("orientation_z", DataType::Float64, false),
                Field::new("lin_vel_x", DataType::Float64, false),
                Field::new("lin_vel_y", DataType::Float64, false),
                Field::new("lin_vel_z", DataType::Float64, false),
                Field::new("ang_vel_x", DataType::Float64, false),
                Field::new("ang_vel_y", DataType::Float64, false),
                Field::new("ang_vel_z", DataType::Float64, false),
                Field::new("timestamp_ns", DataType::Int64, false),
            ]))
        })
        .clone()
}

/// Encode an `Odometry` sample as a single-row `RecordBatch` matching [`schema`].
///
/// # Example
///
/// ```
/// use isaac_sim_arrow::odometry::{Odometry, to_record_batch};
/// let odom = Odometry {
///     chassis_frame_id: "base_link",
///     odom_frame_id: "odom",
///     position_x: 1.0, position_y: 2.0, position_z: 0.0,
///     orientation_w: 1.0, orientation_x: 0.0, orientation_y: 0.0, orientation_z: 0.0,
///     lin_vel_x: 0.4, lin_vel_y: 0.0, lin_vel_z: 0.0,
///     ang_vel_x: 0.0, ang_vel_y: 0.0, ang_vel_z: 0.3,
///     timestamp_ns: 7,
/// };
/// let batch = to_record_batch(&odom).unwrap();
/// assert_eq!(batch.num_rows(), 1);
/// assert_eq!(batch.num_columns(), 16);
/// ```
pub fn to_record_batch(odom: &Odometry) -> Result<RecordBatch, arrow::error::ArrowError> {
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(vec![odom.chassis_frame_id])),
        Arc::new(StringArray::from(vec![odom.odom_frame_id])),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.position_x,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.position_y,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.position_z,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.orientation_w,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.orientation_x,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.orientation_y,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.orientation_z,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.lin_vel_x,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.lin_vel_y,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.lin_vel_z,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.ang_vel_x,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.ang_vel_y,
        ))),
        Arc::new(Float64Array::from_iter_values(std::iter::once(
            odom.ang_vel_z,
        ))),
        Arc::new(Int64Array::from_iter_values(std::iter::once(
            odom.timestamp_ns,
        ))),
    ];
    RecordBatch::try_new(schema(), columns)
}

/// Decode the first row of a `StructArray` into a heap-owned `OdometryOwned`.
///
/// # Example
///
/// ```
/// use arrow::array::StructArray;
/// use isaac_sim_arrow::odometry::{Odometry, to_record_batch, from_struct_array};
/// let odom = Odometry {
///     chassis_frame_id: "base_link",
///     odom_frame_id: "odom",
///     position_x: 1.0, position_y: 2.0, position_z: 0.0,
///     orientation_w: 1.0, orientation_x: 0.0, orientation_y: 0.0, orientation_z: 0.0,
///     lin_vel_x: 0.4, lin_vel_y: 0.0, lin_vel_z: 0.0,
///     ang_vel_x: 0.0, ang_vel_y: 0.0, ang_vel_z: 0.3,
///     timestamp_ns: 7,
/// };
/// let batch = to_record_batch(&odom).unwrap();
/// let array = StructArray::from(batch);
/// let owned = from_struct_array(&array).unwrap();
/// assert_eq!(owned.chassis_frame_id, "base_link");
/// assert_eq!(owned.timestamp_ns, 7);
/// ```
pub fn from_struct_array(array: &StructArray) -> Result<OdometryOwned, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "odometry struct array is empty".into(),
        ));
    }
    let str_at = |idx: usize, name: &str| -> Result<String, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("odometry '{name}' not Utf8"))
            })
            .map(|a| a.value(0).to_string())
    };
    let f64_at = |idx: usize, name: &str| -> Result<f64, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("odometry '{name}' not Float64"))
            })
            .map(|a| a.value(0))
    };
    Ok(OdometryOwned {
        chassis_frame_id: str_at(0, "chassis_frame_id")?,
        odom_frame_id: str_at(1, "odom_frame_id")?,
        position_x: f64_at(2, "position_x")?,
        position_y: f64_at(3, "position_y")?,
        position_z: f64_at(4, "position_z")?,
        orientation_w: f64_at(5, "orientation_w")?,
        orientation_x: f64_at(6, "orientation_x")?,
        orientation_y: f64_at(7, "orientation_y")?,
        orientation_z: f64_at(8, "orientation_z")?,
        lin_vel_x: f64_at(9, "lin_vel_x")?,
        lin_vel_y: f64_at(10, "lin_vel_y")?,
        lin_vel_z: f64_at(11, "lin_vel_z")?,
        ang_vel_x: f64_at(12, "ang_vel_x")?,
        ang_vel_y: f64_at(13, "ang_vel_y")?,
        ang_vel_z: f64_at(14, "ang_vel_z")?,
        timestamp_ns: array
            .column(15)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError("odometry 'timestamp_ns' not Int64".into())
            })?
            .value(0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn round_trips_through_record_batch() {
        let odom = Odometry {
            chassis_frame_id: "base_link",
            odom_frame_id: "odom",
            position_x: 1.0,
            position_y: 2.0,
            position_z: 0.0,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            lin_vel_x: 0.4,
            lin_vel_y: 0.0,
            lin_vel_z: 0.0,
            ang_vel_x: 0.0,
            ang_vel_y: 0.0,
            ang_vel_z: 0.3,
            timestamp_ns: 7,
        };
        let batch = to_record_batch(&odom).expect("convert");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 16);

        let chassis = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("chassis_frame_id is Utf8");
        assert_eq!(chassis.value(0), "base_link");

        let lin_x = batch
            .column(9)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lin_vel_x is Float64");
        assert!((lin_x.value(0) - 0.4).abs() < 1e-9);
    }

    #[test]
    fn from_struct_array_round_trips() {
        let odom = Odometry {
            chassis_frame_id: "base_link",
            odom_frame_id: "odom",
            position_x: 1.0,
            position_y: 2.0,
            position_z: 0.0,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            lin_vel_x: 0.4,
            lin_vel_y: 0.0,
            lin_vel_z: 0.0,
            ang_vel_x: 0.0,
            ang_vel_y: 0.0,
            ang_vel_z: 0.3,
            timestamp_ns: 7,
        };
        let batch = to_record_batch(&odom).expect("to");
        let array = StructArray::from(batch);
        let owned = from_struct_array(&array).expect("from");
        assert_eq!(owned.chassis_frame_id, "base_link");
        assert_eq!(owned.odom_frame_id, "odom");
        assert!((owned.lin_vel_x - 0.4).abs() < 1e-9);
        assert_eq!(owned.timestamp_ns, 7);
    }
}

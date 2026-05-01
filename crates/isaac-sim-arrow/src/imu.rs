use std::sync::{Arc, OnceLock};

use arrow::array::{Array, ArrayRef, Float64Array, Int64Array, StringArray, StructArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

pub struct Imu<'a> {
    pub frame_id: &'a str,
    pub lin_acc_x: f64,
    pub lin_acc_y: f64,
    pub lin_acc_z: f64,
    pub ang_vel_x: f64,
    pub ang_vel_y: f64,
    pub ang_vel_z: f64,
    pub orientation_w: f64,
    pub orientation_x: f64,
    pub orientation_y: f64,
    pub orientation_z: f64,
    pub timestamp_ns: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImuOwned {
    pub frame_id: String,
    pub lin_acc_x: f64,
    pub lin_acc_y: f64,
    pub lin_acc_z: f64,
    pub ang_vel_x: f64,
    pub ang_vel_y: f64,
    pub ang_vel_z: f64,
    pub orientation_w: f64,
    pub orientation_x: f64,
    pub orientation_y: f64,
    pub orientation_z: f64,
    pub timestamp_ns: i64,
}

pub fn schema() -> SchemaRef {
    static SCHEMA: OnceLock<SchemaRef> = OnceLock::new();
    SCHEMA
        .get_or_init(|| {
            Arc::new(Schema::new(vec![
                Field::new("frame_id", DataType::Utf8, false),
                Field::new("lin_acc_x", DataType::Float64, false),
                Field::new("lin_acc_y", DataType::Float64, false),
                Field::new("lin_acc_z", DataType::Float64, false),
                Field::new("ang_vel_x", DataType::Float64, false),
                Field::new("ang_vel_y", DataType::Float64, false),
                Field::new("ang_vel_z", DataType::Float64, false),
                Field::new("orientation_w", DataType::Float64, false),
                Field::new("orientation_x", DataType::Float64, false),
                Field::new("orientation_y", DataType::Float64, false),
                Field::new("orientation_z", DataType::Float64, false),
                Field::new("timestamp_ns", DataType::Int64, false),
            ]))
        })
        .clone()
}

pub fn to_record_batch(imu: &Imu) -> Result<RecordBatch, arrow::error::ArrowError> {
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(vec![imu.frame_id])),
        Arc::new(Float64Array::from(vec![imu.lin_acc_x])),
        Arc::new(Float64Array::from(vec![imu.lin_acc_y])),
        Arc::new(Float64Array::from(vec![imu.lin_acc_z])),
        Arc::new(Float64Array::from(vec![imu.ang_vel_x])),
        Arc::new(Float64Array::from(vec![imu.ang_vel_y])),
        Arc::new(Float64Array::from(vec![imu.ang_vel_z])),
        Arc::new(Float64Array::from(vec![imu.orientation_w])),
        Arc::new(Float64Array::from(vec![imu.orientation_x])),
        Arc::new(Float64Array::from(vec![imu.orientation_y])),
        Arc::new(Float64Array::from(vec![imu.orientation_z])),
        Arc::new(Int64Array::from(vec![imu.timestamp_ns])),
    ];
    RecordBatch::try_new(schema(), columns)
}

pub fn from_struct_array(array: &StructArray) -> Result<ImuOwned, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "imu struct array is empty".into(),
        ));
    }
    let f64_at = |idx: usize, name: &str| -> Result<f64, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("imu '{name}' not Float64"))
            })
            .map(|a| a.value(0))
    };
    let frame_id = array
        .column(0)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| arrow::error::ArrowError::SchemaError("imu 'frame_id' not Utf8".into()))?
        .value(0)
        .to_string();
    let timestamp_ns = array
        .column(11)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("imu 'timestamp_ns' not Int64".into())
        })?
        .value(0);
    Ok(ImuOwned {
        frame_id,
        lin_acc_x: f64_at(1, "lin_acc_x")?,
        lin_acc_y: f64_at(2, "lin_acc_y")?,
        lin_acc_z: f64_at(3, "lin_acc_z")?,
        ang_vel_x: f64_at(4, "ang_vel_x")?,
        ang_vel_y: f64_at(5, "ang_vel_y")?,
        ang_vel_z: f64_at(6, "ang_vel_z")?,
        orientation_w: f64_at(7, "orientation_w")?,
        orientation_x: f64_at(8, "orientation_x")?,
        orientation_y: f64_at(9, "orientation_y")?,
        orientation_z: f64_at(10, "orientation_z")?,
        timestamp_ns,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn round_trips_through_record_batch() {
        let imu = Imu {
            frame_id: "sim_imu",
            lin_acc_x: 0.1,
            lin_acc_y: 0.2,
            lin_acc_z: 9.81,
            ang_vel_x: 0.0,
            ang_vel_y: 0.0,
            ang_vel_z: 0.5,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            timestamp_ns: 12345,
        };
        let batch = to_record_batch(&imu).expect("convert");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 12);

        let frame = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("frame_id is Utf8");
        assert_eq!(frame.value(0), "sim_imu");

        let lin_z = batch
            .column(3)
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("lin_acc_z is Float64");
        assert!((lin_z.value(0) - 9.81).abs() < 1e-9);

        let ts = batch
            .column(11)
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("timestamp_ns is Int64");
        assert_eq!(ts.value(0), 12345);
    }

    #[test]
    fn from_struct_array_round_trips() {
        let imu = Imu {
            frame_id: "sim_imu",
            lin_acc_x: 0.1,
            lin_acc_y: 0.2,
            lin_acc_z: 9.81,
            ang_vel_x: 0.0,
            ang_vel_y: 0.0,
            ang_vel_z: 0.5,
            orientation_w: 1.0,
            orientation_x: 0.0,
            orientation_y: 0.0,
            orientation_z: 0.0,
            timestamp_ns: 12345,
        };
        let batch = to_record_batch(&imu).expect("to");
        let array = StructArray::from(batch);
        let owned = from_struct_array(&array).expect("from");
        assert_eq!(owned.frame_id, "sim_imu");
        assert!((owned.lin_acc_z - 9.81).abs() < 1e-9);
        assert_eq!(owned.timestamp_ns, 12345);
    }
}

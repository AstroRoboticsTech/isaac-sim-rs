use std::sync::{Arc, OnceLock};

use arrow::array::{Array, ArrayRef, Float32Array, Int64Array, StructArray};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CmdVel {
    pub linear_x: f32,
    pub linear_y: f32,
    pub linear_z: f32,
    pub angular_x: f32,
    pub angular_y: f32,
    pub angular_z: f32,
    pub timestamp_ns: i64,
}

impl Default for CmdVel {
    fn default() -> Self {
        Self {
            linear_x: 0.0,
            linear_y: 0.0,
            linear_z: 0.0,
            angular_x: 0.0,
            angular_y: 0.0,
            angular_z: 0.0,
            timestamp_ns: 0,
        }
    }
}

pub fn schema() -> SchemaRef {
    static SCHEMA: OnceLock<SchemaRef> = OnceLock::new();
    SCHEMA
        .get_or_init(|| {
            Arc::new(Schema::new(vec![
                Field::new("linear_x", DataType::Float32, false),
                Field::new("linear_y", DataType::Float32, false),
                Field::new("linear_z", DataType::Float32, false),
                Field::new("angular_x", DataType::Float32, false),
                Field::new("angular_y", DataType::Float32, false),
                Field::new("angular_z", DataType::Float32, false),
                Field::new("timestamp_ns", DataType::Int64, false),
            ]))
        })
        .clone()
}

pub fn to_record_batch(twist: &CmdVel) -> Result<RecordBatch, arrow::error::ArrowError> {
    let columns: Vec<ArrayRef> = vec![
        Arc::new(Float32Array::from(vec![twist.linear_x])),
        Arc::new(Float32Array::from(vec![twist.linear_y])),
        Arc::new(Float32Array::from(vec![twist.linear_z])),
        Arc::new(Float32Array::from(vec![twist.angular_x])),
        Arc::new(Float32Array::from(vec![twist.angular_y])),
        Arc::new(Float32Array::from(vec![twist.angular_z])),
        Arc::new(Int64Array::from(vec![twist.timestamp_ns])),
    ];
    RecordBatch::try_new(schema(), columns)
}

/// Decode a single CmdVel sample from a `StructArray` whose fields
/// match [`schema`]. Returns the first row; errors on field mismatch
/// or empty input. Symmetric to [`to_record_batch`].
pub fn from_struct_array(array: &StructArray) -> Result<CmdVel, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "cmd_vel struct array is empty".into(),
        ));
    }
    let schema = schema();
    let names = schema.fields().iter().map(|f| f.name().clone());
    let mut out = CmdVel::default();
    for (idx, name) in names.enumerate() {
        let col = array.column(idx);
        match name.as_str() {
            "linear_x" => out.linear_x = col_f32(col, "linear_x")?,
            "linear_y" => out.linear_y = col_f32(col, "linear_y")?,
            "linear_z" => out.linear_z = col_f32(col, "linear_z")?,
            "angular_x" => out.angular_x = col_f32(col, "angular_x")?,
            "angular_y" => out.angular_y = col_f32(col, "angular_y")?,
            "angular_z" => out.angular_z = col_f32(col, "angular_z")?,
            "timestamp_ns" => out.timestamp_ns = col_i64(col, "timestamp_ns")?,
            other => {
                return Err(arrow::error::ArrowError::SchemaError(format!(
                    "unexpected cmd_vel column '{other}'"
                )));
            }
        }
    }
    Ok(out)
}

fn col_f32(col: &ArrayRef, name: &str) -> Result<f32, arrow::error::ArrowError> {
    col.as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError(format!("cmd_vel '{name}' not Float32"))
        })
        .map(|a| a.value(0))
}

fn col_i64(col: &ArrayRef, name: &str) -> Result<i64, arrow::error::ArrowError> {
    col.as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| arrow::error::ArrowError::SchemaError(format!("cmd_vel '{name}' not Int64")))
        .map(|a| a.value(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_struct_array() {
        let twist = CmdVel {
            linear_x: 0.4,
            linear_y: 0.0,
            linear_z: 0.0,
            angular_x: 0.0,
            angular_y: 0.0,
            angular_z: 0.3,
            timestamp_ns: 999,
        };
        let batch = to_record_batch(&twist).expect("convert");
        let array = StructArray::from(batch);
        let decoded = from_struct_array(&array).expect("decode");
        assert_eq!(decoded, twist);
    }
}

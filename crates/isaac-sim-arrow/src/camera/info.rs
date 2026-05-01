use std::sync::{Arc, OnceLock};

use arrow::array::{
    Array, ArrayRef, Float32Array, Float64Array, Int32Array, Int64Array, ListArray, StringArray,
    StructArray,
};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

pub struct CameraInfo<'a> {
    pub frame_id: &'a str,
    pub distortion_model: &'a str,
    pub projection_type: &'a str,
    pub k: &'a [f64],
    pub r: &'a [f64],
    pub p: &'a [f64],
    pub distortion: &'a [f32],
    pub width: i32,
    pub height: i32,
    pub timestamp_ns: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraInfoOwned {
    pub frame_id: String,
    pub distortion_model: String,
    pub projection_type: String,
    pub k: Vec<f64>,
    pub r: Vec<f64>,
    pub p: Vec<f64>,
    pub distortion: Vec<f32>,
    pub width: i32,
    pub height: i32,
    pub timestamp_ns: i64,
}

pub fn schema() -> SchemaRef {
    static SCHEMA: OnceLock<SchemaRef> = OnceLock::new();
    SCHEMA
        .get_or_init(|| {
            Arc::new(Schema::new(vec![
                Field::new("frame_id", DataType::Utf8, false),
                Field::new("distortion_model", DataType::Utf8, false),
                Field::new("projection_type", DataType::Utf8, false),
                Field::new(
                    "k",
                    DataType::List(Arc::new(Field::new("item", DataType::Float64, false))),
                    false,
                ),
                Field::new(
                    "r",
                    DataType::List(Arc::new(Field::new("item", DataType::Float64, false))),
                    false,
                ),
                Field::new(
                    "p",
                    DataType::List(Arc::new(Field::new("item", DataType::Float64, false))),
                    false,
                ),
                Field::new(
                    "distortion",
                    DataType::List(Arc::new(Field::new("item", DataType::Float32, false))),
                    false,
                ),
                Field::new("width", DataType::Int32, false),
                Field::new("height", DataType::Int32, false),
                Field::new("timestamp_ns", DataType::Int64, false),
            ]))
        })
        .clone()
}

fn list_f64(values: &[f64]) -> ListArray {
    let inner = Float64Array::from_iter_values(values.iter().copied());
    let offsets = OffsetBuffer::from_lengths([values.len()]);
    ListArray::new(
        Arc::new(Field::new("item", DataType::Float64, false)),
        offsets,
        Arc::new(inner),
        None,
    )
}

fn list_f32(values: &[f32]) -> ListArray {
    let inner = Float32Array::from_iter_values(values.iter().copied());
    let offsets = OffsetBuffer::from_lengths([values.len()]);
    ListArray::new(
        Arc::new(Field::new("item", DataType::Float32, false)),
        offsets,
        Arc::new(inner),
        None,
    )
}

pub fn to_record_batch(info: &CameraInfo) -> Result<RecordBatch, arrow::error::ArrowError> {
    let columns: Vec<ArrayRef> = vec![
        Arc::new(StringArray::from(vec![info.frame_id])),
        Arc::new(StringArray::from(vec![info.distortion_model])),
        Arc::new(StringArray::from(vec![info.projection_type])),
        Arc::new(list_f64(info.k)),
        Arc::new(list_f64(info.r)),
        Arc::new(list_f64(info.p)),
        Arc::new(list_f32(info.distortion)),
        Arc::new(Int32Array::from(vec![info.width])),
        Arc::new(Int32Array::from(vec![info.height])),
        Arc::new(Int64Array::from(vec![info.timestamp_ns])),
    ];
    RecordBatch::try_new(schema(), columns)
}

pub fn from_struct_array(array: &StructArray) -> Result<CameraInfoOwned, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "camera_info struct array is empty".into(),
        ));
    }
    let str_at = |idx: usize, name: &str| -> Result<String, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<StringArray>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("camera_info '{name}' not Utf8"))
            })
            .map(|a| a.value(0).to_string())
    };
    let f64_list = |idx: usize, name: &str| -> Result<Vec<f64>, arrow::error::ArrowError> {
        let list = array
            .column(idx)
            .as_any()
            .downcast_ref::<ListArray>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("camera_info '{name}' not ListArray"))
            })?;
        Ok(list
            .values()
            .as_any()
            .downcast_ref::<Float64Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!(
                    "camera_info '{name}' inner not Float64"
                ))
            })?
            .values()
            .to_vec())
    };
    let f32_list = |idx: usize, name: &str| -> Result<Vec<f32>, arrow::error::ArrowError> {
        let list = array
            .column(idx)
            .as_any()
            .downcast_ref::<ListArray>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("camera_info '{name}' not ListArray"))
            })?;
        Ok(list
            .values()
            .as_any()
            .downcast_ref::<Float32Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!(
                    "camera_info '{name}' inner not Float32"
                ))
            })?
            .values()
            .to_vec())
    };
    Ok(CameraInfoOwned {
        frame_id: str_at(0, "frame_id")?,
        distortion_model: str_at(1, "distortion_model")?,
        projection_type: str_at(2, "projection_type")?,
        k: f64_list(3, "k")?,
        r: f64_list(4, "r")?,
        p: f64_list(5, "p")?,
        distortion: f32_list(6, "distortion")?,
        width: array
            .column(7)
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError("camera_info 'width' not Int32".into())
            })?
            .value(0),
        height: array
            .column(8)
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError("camera_info 'height' not Int32".into())
            })?
            .value(0),
        timestamp_ns: array
            .column(9)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError("camera_info 'timestamp_ns' not Int64".into())
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
        let k = [500.0_f64, 0.0, 320.0, 0.0, 500.0, 240.0, 0.0, 0.0, 1.0];
        let r = [1.0_f64, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let p = [
            500.0_f64, 0.0, 320.0, 0.0, 0.0, 500.0, 240.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        ];
        let d = [0.0_f32; 5];
        let info = CameraInfo {
            frame_id: "sim_camera",
            distortion_model: "plumb_bob",
            projection_type: "pinhole",
            k: &k,
            r: &r,
            p: &p,
            distortion: &d,
            width: 640,
            height: 480,
            timestamp_ns: 7,
        };
        let batch = to_record_batch(&info).expect("convert");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 10);

        let frame = batch
            .column(0)
            .as_any()
            .downcast_ref::<StringArray>()
            .expect("frame_id is Utf8");
        assert_eq!(frame.value(0), "sim_camera");

        let k_col = batch
            .column(3)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("k is ListArray");
        let k_inner = k_col
            .values()
            .as_any()
            .downcast_ref::<Float64Array>()
            .expect("k inner is Float64");
        assert_eq!(k_inner.len(), 9);
        assert_eq!(k_inner.value(0), 500.0);

        let p_col = batch
            .column(5)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("p is ListArray");
        assert_eq!(p_col.values().len(), 12);
    }

    #[test]
    fn from_struct_array_round_trips() {
        let k = [500.0_f64, 0.0, 320.0, 0.0, 500.0, 240.0, 0.0, 0.0, 1.0];
        let r = [1.0_f64, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let p = [
            500.0_f64, 0.0, 320.0, 0.0, 0.0, 500.0, 240.0, 0.0, 0.0, 0.0, 1.0, 0.0,
        ];
        let d = [0.0_f32; 5];
        let info = CameraInfo {
            frame_id: "sim_camera",
            distortion_model: "plumb_bob",
            projection_type: "pinhole",
            k: &k,
            r: &r,
            p: &p,
            distortion: &d,
            width: 640,
            height: 480,
            timestamp_ns: 7,
        };
        let batch = to_record_batch(&info).expect("to");
        let array = StructArray::from(batch);
        let owned = from_struct_array(&array).expect("from");
        assert_eq!(owned.frame_id, "sim_camera");
        assert_eq!(owned.distortion_model, "plumb_bob");
        assert_eq!(owned.k, k);
        assert_eq!(owned.p, p);
        assert_eq!(owned.distortion, d);
        assert_eq!(owned.width, 640);
        assert_eq!(owned.timestamp_ns, 7);
    }
}

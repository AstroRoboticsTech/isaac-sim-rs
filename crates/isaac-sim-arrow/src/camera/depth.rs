use std::sync::{Arc, OnceLock};

use arrow::array::{Array, ArrayRef, Float32Array, Int32Array, Int64Array, ListArray, StructArray};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

pub struct CameraDepth<'a> {
    pub depths: &'a [f32],
    pub width: i32,
    pub height: i32,
    pub fx: f32,
    pub fy: f32,
    pub cx: f32,
    pub cy: f32,
    pub timestamp_ns: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraDepthOwned {
    pub depths: Vec<f32>,
    pub width: i32,
    pub height: i32,
    pub fx: f32,
    pub fy: f32,
    pub cx: f32,
    pub cy: f32,
    pub timestamp_ns: i64,
}

pub fn schema() -> SchemaRef {
    static SCHEMA: OnceLock<SchemaRef> = OnceLock::new();
    SCHEMA
        .get_or_init(|| {
            Arc::new(Schema::new(vec![
                Field::new(
                    "depths",
                    DataType::List(Arc::new(Field::new("item", DataType::Float32, false))),
                    false,
                ),
                Field::new("width", DataType::Int32, false),
                Field::new("height", DataType::Int32, false),
                Field::new("fx", DataType::Float32, false),
                Field::new("fy", DataType::Float32, false),
                Field::new("cx", DataType::Float32, false),
                Field::new("cy", DataType::Float32, false),
                Field::new("timestamp_ns", DataType::Int64, false),
            ]))
        })
        .clone()
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

pub fn to_record_batch(img: &CameraDepth) -> Result<RecordBatch, arrow::error::ArrowError> {
    let columns: Vec<ArrayRef> = vec![
        Arc::new(list_f32(img.depths)),
        Arc::new(Int32Array::from_iter_values(std::iter::once(img.width))),
        Arc::new(Int32Array::from_iter_values(std::iter::once(img.height))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(img.fx))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(img.fy))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(img.cx))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(img.cy))),
        Arc::new(Int64Array::from_iter_values(std::iter::once(
            img.timestamp_ns,
        ))),
    ];
    RecordBatch::try_new(schema(), columns)
}

pub struct CameraDepthBorrowed<'a> {
    pub depths: &'a [f32],
    pub width: i32,
    pub height: i32,
    pub fx: f32,
    pub fy: f32,
    pub cx: f32,
    pub cy: f32,
    pub timestamp_ns: i64,
}

pub fn from_struct_array_borrowed(
    array: &StructArray,
) -> Result<CameraDepthBorrowed<'_>, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "camera_depth struct array is empty".into(),
        ));
    }
    let depths_list = array
        .column(0)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("camera_depth 'depths' not ListArray".into())
        })?;
    let depths = depths_list
        .values()
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("camera_depth 'depths' inner not Float32".into())
        })?
        .values();
    let i32_at = |idx: usize, name: &str| -> Result<i32, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("camera_depth '{name}' not Int32"))
            })
            .map(|a| a.value(0))
    };
    let f32_at = |idx: usize, name: &str| -> Result<f32, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<Float32Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("camera_depth '{name}' not Float32"))
            })
            .map(|a| a.value(0))
    };
    Ok(CameraDepthBorrowed {
        depths,
        width: i32_at(1, "width")?,
        height: i32_at(2, "height")?,
        fx: f32_at(3, "fx")?,
        fy: f32_at(4, "fy")?,
        cx: f32_at(5, "cx")?,
        cy: f32_at(6, "cy")?,
        timestamp_ns: array
            .column(7)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(
                    "camera_depth 'timestamp_ns' not Int64".into(),
                )
            })?
            .value(0),
    })
}

pub fn from_struct_array(
    array: &StructArray,
) -> Result<CameraDepthOwned, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "camera_depth struct array is empty".into(),
        ));
    }
    let depths_list = array
        .column(0)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("camera_depth 'depths' not ListArray".into())
        })?;
    let depths = depths_list
        .values()
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("camera_depth 'depths' inner not Float32".into())
        })?
        .values()
        .to_vec();
    let i32_at = |idx: usize, name: &str| -> Result<i32, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("camera_depth '{name}' not Int32"))
            })
            .map(|a| a.value(0))
    };
    let f32_at = |idx: usize, name: &str| -> Result<f32, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<Float32Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("camera_depth '{name}' not Float32"))
            })
            .map(|a| a.value(0))
    };
    Ok(CameraDepthOwned {
        depths,
        width: i32_at(1, "width")?,
        height: i32_at(2, "height")?,
        fx: f32_at(3, "fx")?,
        fy: f32_at(4, "fy")?,
        cx: f32_at(5, "cx")?,
        cy: f32_at(6, "cy")?,
        timestamp_ns: array
            .column(7)
            .as_any()
            .downcast_ref::<Int64Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(
                    "camera_depth 'timestamp_ns' not Int64".into(),
                )
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
        let depths = vec![0.5_f32; 4]; // 2x2
        let img = CameraDepth {
            depths: &depths,
            width: 2,
            height: 2,
            fx: 100.0,
            fy: 100.0,
            cx: 1.0,
            cy: 1.0,
            timestamp_ns: 99,
        };
        let batch = to_record_batch(&img).expect("convert");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 8);

        let depths_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("depths is ListArray");
        let inner = depths_col
            .values()
            .as_any()
            .downcast_ref::<Float32Array>()
            .expect("inner is Float32Array");
        assert_eq!(inner.len(), 4);

        let ts = batch
            .column(7)
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("timestamp_ns is Int64");
        assert_eq!(ts.value(0), 99);
    }

    #[test]
    fn from_struct_array_round_trips() {
        let depths = vec![0.5_f32, 1.0, 1.5, 2.0];
        let img = CameraDepth {
            depths: &depths,
            width: 2,
            height: 2,
            fx: 100.0,
            fy: 100.0,
            cx: 1.0,
            cy: 1.0,
            timestamp_ns: 99,
        };
        let batch = to_record_batch(&img).expect("to");
        let array = StructArray::from(batch);
        let owned = from_struct_array(&array).expect("from");
        assert_eq!(owned.depths, depths);
        assert_eq!(owned.width, 2);
        assert_eq!(owned.timestamp_ns, 99);
    }
}

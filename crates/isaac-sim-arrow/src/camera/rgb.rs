use std::sync::{Arc, OnceLock};

use arrow::array::{
    Array, ArrayRef, Float32Array, Int32Array, Int64Array, ListArray, StructArray, UInt8Array,
};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

pub struct CameraRgb<'a> {
    pub pixels: &'a [u8],
    pub width: i32,
    pub height: i32,
    pub fx: f32,
    pub fy: f32,
    pub cx: f32,
    pub cy: f32,
    pub timestamp_ns: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraRgbOwned {
    pub pixels: Vec<u8>,
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
                    "pixels",
                    DataType::List(Arc::new(Field::new("item", DataType::UInt8, false))),
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

fn list_u8(values: &[u8]) -> ListArray {
    let inner = UInt8Array::from_iter_values(values.iter().copied());
    let offsets = OffsetBuffer::from_lengths([values.len()]);
    ListArray::new(
        Arc::new(Field::new("item", DataType::UInt8, false)),
        offsets,
        Arc::new(inner),
        None,
    )
}

pub fn to_record_batch(img: &CameraRgb) -> Result<RecordBatch, arrow::error::ArrowError> {
    let columns: Vec<ArrayRef> = vec![
        Arc::new(list_u8(img.pixels)),
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

pub struct CameraRgbBorrowed<'a> {
    pub pixels: &'a [u8],
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
) -> Result<CameraRgbBorrowed<'_>, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "camera_rgb struct array is empty".into(),
        ));
    }
    let pixels_list = array
        .column(0)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("camera_rgb 'pixels' not ListArray".into())
        })?;
    let pixels = pixels_list
        .values()
        .as_any()
        .downcast_ref::<UInt8Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("camera_rgb 'pixels' inner not UInt8".into())
        })?
        .values();
    Ok(CameraRgbBorrowed {
        pixels,
        width: scalar_i32(array, 1, "width")?,
        height: scalar_i32(array, 2, "height")?,
        fx: scalar_f32(array, 3, "fx")?,
        fy: scalar_f32(array, 4, "fy")?,
        cx: scalar_f32(array, 5, "cx")?,
        cy: scalar_f32(array, 6, "cy")?,
        timestamp_ns: scalar_i64(array, 7, "timestamp_ns")?,
    })
}

pub fn from_struct_array(array: &StructArray) -> Result<CameraRgbOwned, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "camera_rgb struct array is empty".into(),
        ));
    }
    let pixels_list = array
        .column(0)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("camera_rgb 'pixels' not ListArray".into())
        })?;
    let pixels = pixels_list
        .values()
        .as_any()
        .downcast_ref::<UInt8Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("camera_rgb 'pixels' inner not UInt8".into())
        })?
        .values()
        .to_vec();
    Ok(CameraRgbOwned {
        pixels,
        width: scalar_i32(array, 1, "width")?,
        height: scalar_i32(array, 2, "height")?,
        fx: scalar_f32(array, 3, "fx")?,
        fy: scalar_f32(array, 4, "fy")?,
        cx: scalar_f32(array, 5, "cx")?,
        cy: scalar_f32(array, 6, "cy")?,
        timestamp_ns: scalar_i64(array, 7, "timestamp_ns")?,
    })
}

fn scalar_i32(
    array: &StructArray,
    idx: usize,
    name: &str,
) -> Result<i32, arrow::error::ArrowError> {
    array
        .column(idx)
        .as_any()
        .downcast_ref::<Int32Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError(format!("camera_rgb '{name}' not Int32"))
        })
        .map(|a| a.value(0))
}

fn scalar_f32(
    array: &StructArray,
    idx: usize,
    name: &str,
) -> Result<f32, arrow::error::ArrowError> {
    array
        .column(idx)
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError(format!("camera_rgb '{name}' not Float32"))
        })
        .map(|a| a.value(0))
}

fn scalar_i64(
    array: &StructArray,
    idx: usize,
    name: &str,
) -> Result<i64, arrow::error::ArrowError> {
    array
        .column(idx)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError(format!("camera_rgb '{name}' not Int64"))
        })
        .map(|a| a.value(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::Array;

    #[test]
    fn round_trips_through_record_batch() {
        let pixels = vec![0_u8; 12]; // 2x2 RGB
        let img = CameraRgb {
            pixels: &pixels,
            width: 2,
            height: 2,
            fx: 100.0,
            fy: 100.0,
            cx: 1.0,
            cy: 1.0,
            timestamp_ns: 42,
        };
        let batch = to_record_batch(&img).expect("convert");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 8);

        let pixels_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("pixels is ListArray");
        let inner = pixels_col
            .values()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("inner is UInt8Array");
        assert_eq!(inner.len(), 12);

        let ts = batch
            .column(7)
            .as_any()
            .downcast_ref::<Int64Array>()
            .expect("timestamp_ns is Int64");
        assert_eq!(ts.value(0), 42);
    }

    #[test]
    fn from_struct_array_round_trips() {
        let pixels = vec![0_u8, 64, 128, 255, 1, 2, 3, 4, 5, 6, 7, 8];
        let img = CameraRgb {
            pixels: &pixels,
            width: 2,
            height: 2,
            fx: 100.0,
            fy: 110.0,
            cx: 1.0,
            cy: 2.0,
            timestamp_ns: 42,
        };
        let batch = to_record_batch(&img).expect("to");
        let array = StructArray::from(batch);
        let owned = from_struct_array(&array).expect("from");
        assert_eq!(owned.pixels, pixels);
        assert_eq!(owned.width, 2);
        assert_eq!(owned.fy, 110.0);
        assert_eq!(owned.timestamp_ns, 42);
    }
}

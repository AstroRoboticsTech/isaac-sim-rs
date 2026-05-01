use std::sync::{Arc, OnceLock};

use arrow::array::{Array, ArrayRef, Float32Array, Int32Array, ListArray, StructArray};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

pub struct LidarPointCloud<'a> {
    pub points: &'a [f32],
    pub num_points: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LidarPointCloudOwned {
    pub points: Vec<f32>,
    pub num_points: i32,
    pub width: i32,
    pub height: i32,
}

pub fn schema() -> SchemaRef {
    static SCHEMA: OnceLock<SchemaRef> = OnceLock::new();
    SCHEMA
        .get_or_init(|| {
            Arc::new(Schema::new(vec![
                Field::new(
                    "points",
                    DataType::List(Arc::new(Field::new("item", DataType::Float32, false))),
                    false,
                ),
                Field::new("num_points", DataType::Int32, false),
                Field::new("width", DataType::Int32, false),
                Field::new("height", DataType::Int32, false),
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

pub fn to_record_batch(pc: &LidarPointCloud) -> Result<RecordBatch, arrow::error::ArrowError> {
    let columns: Vec<ArrayRef> = vec![
        Arc::new(list_f32(pc.points)),
        Arc::new(Int32Array::from_iter_values(std::iter::once(pc.num_points))),
        Arc::new(Int32Array::from_iter_values(std::iter::once(pc.width))),
        Arc::new(Int32Array::from_iter_values(std::iter::once(pc.height))),
    ];
    RecordBatch::try_new(schema(), columns)
}

pub struct LidarPointCloudBorrowed<'a> {
    pub points: &'a [f32],
    pub num_points: i32,
    pub width: i32,
    pub height: i32,
}

pub fn from_struct_array_borrowed(
    array: &StructArray,
) -> Result<LidarPointCloudBorrowed<'_>, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "lidar_pointcloud struct array is empty".into(),
        ));
    }
    let pts_list = array
        .column(0)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("pointcloud 'points' not ListArray".into())
        })?;
    let points = pts_list
        .values()
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("pointcloud 'points' inner not Float32".into())
        })?
        .values();
    let i32_at = |idx: usize, name: &str| -> Result<i32, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("pointcloud '{name}' not Int32"))
            })
            .map(|a| a.value(0))
    };
    Ok(LidarPointCloudBorrowed {
        points,
        num_points: i32_at(1, "num_points")?,
        width: i32_at(2, "width")?,
        height: i32_at(3, "height")?,
    })
}

pub fn from_struct_array(
    array: &StructArray,
) -> Result<LidarPointCloudOwned, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "lidar_pointcloud struct array is empty".into(),
        ));
    }
    let pts_list = array
        .column(0)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("pointcloud 'points' not ListArray".into())
        })?;
    let pts_inner = pts_list
        .values()
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError("pointcloud 'points' inner not Float32".into())
        })?;
    let i32_at = |idx: usize, name: &str| -> Result<i32, arrow::error::ArrowError> {
        array
            .column(idx)
            .as_any()
            .downcast_ref::<Int32Array>()
            .ok_or_else(|| {
                arrow::error::ArrowError::SchemaError(format!("pointcloud '{name}' not Int32"))
            })
            .map(|a| a.value(0))
    };
    Ok(LidarPointCloudOwned {
        points: pts_inner.values().to_vec(),
        num_points: i32_at(1, "num_points")?,
        width: i32_at(2, "width")?,
        height: i32_at(3, "height")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_record_batch() {
        let points = [
            1.0_f32, 0.0, 0.0, //
            0.0, 1.0, 0.0, //
            0.0, 0.0, 1.0,
        ];
        let pc = LidarPointCloud {
            points: &points,
            num_points: 3,
            width: 3,
            height: 1,
        };
        let batch = to_record_batch(&pc).expect("convert");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 4);

        let pts_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("points is ListArray");
        let inner = pts_col
            .values()
            .as_any()
            .downcast_ref::<Float32Array>()
            .expect("inner is Float32Array");
        assert_eq!(inner.len(), 9);
        assert_eq!(inner.value(0), 1.0);
        assert_eq!(inner.value(8), 1.0);
    }

    #[test]
    fn from_struct_array_round_trips() {
        let points = [1.0_f32, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        let pc = LidarPointCloud {
            points: &points,
            num_points: 3,
            width: 3,
            height: 1,
        };
        let batch = to_record_batch(&pc).expect("to");
        let array = StructArray::from(batch);
        let owned = from_struct_array(&array).expect("from");
        assert_eq!(owned.points, points);
        assert_eq!(owned.num_points, 3);
    }
}

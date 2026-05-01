use std::sync::{Arc, OnceLock};

use arrow::array::{Array, ArrayRef, Float32Array, Int32Array, ListArray, StructArray, UInt8Array};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

pub struct LidarFlatScan<'a> {
    pub depths: &'a [f32],
    pub intensities: &'a [u8],
    pub horizontal_fov: f32,
    pub horizontal_resolution: f32,
    pub azimuth_min: f32,
    pub azimuth_max: f32,
    pub depth_min: f32,
    pub depth_max: f32,
    pub num_rows: i32,
    pub num_cols: i32,
    pub rotation_rate: f32,
}

/// Owned variant returned by [`from_struct_array`]. Holds heap-owned
/// payload so a downstream dora node can keep the decoded value across
/// the next event without a borrow on the inbound `ArrayRef`.
#[derive(Debug, Clone, PartialEq)]
pub struct LidarFlatScanOwned {
    pub depths: Vec<f32>,
    pub intensities: Vec<u8>,
    pub horizontal_fov: f32,
    pub horizontal_resolution: f32,
    pub azimuth_min: f32,
    pub azimuth_max: f32,
    pub depth_min: f32,
    pub depth_max: f32,
    pub num_rows: i32,
    pub num_cols: i32,
    pub rotation_rate: f32,
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
                Field::new(
                    "intensities",
                    DataType::List(Arc::new(Field::new("item", DataType::UInt8, false))),
                    false,
                ),
                Field::new("horizontal_fov", DataType::Float32, false),
                Field::new("horizontal_resolution", DataType::Float32, false),
                Field::new("azimuth_min", DataType::Float32, false),
                Field::new("azimuth_max", DataType::Float32, false),
                Field::new("depth_min", DataType::Float32, false),
                Field::new("depth_max", DataType::Float32, false),
                Field::new("num_rows", DataType::Int32, false),
                Field::new("num_cols", DataType::Int32, false),
                Field::new("rotation_rate", DataType::Float32, false),
            ]))
        })
        .clone()
}

pub fn to_record_batch(scan: &LidarFlatScan) -> Result<RecordBatch, arrow::error::ArrowError> {
    let depths_inner = Float32Array::from_iter_values(scan.depths.iter().copied());
    let depths_offsets = OffsetBuffer::from_lengths([scan.depths.len()]);
    let depths = ListArray::new(
        Arc::new(Field::new("item", DataType::Float32, false)),
        depths_offsets,
        Arc::new(depths_inner),
        None,
    );

    let intensities_inner = UInt8Array::from_iter_values(scan.intensities.iter().copied());
    let intensities_offsets = OffsetBuffer::from_lengths([scan.intensities.len()]);
    let intensities = ListArray::new(
        Arc::new(Field::new("item", DataType::UInt8, false)),
        intensities_offsets,
        Arc::new(intensities_inner),
        None,
    );

    let columns: Vec<ArrayRef> = vec![
        Arc::new(depths),
        Arc::new(intensities),
        Arc::new(Float32Array::from_iter_values(std::iter::once(
            scan.horizontal_fov,
        ))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(
            scan.horizontal_resolution,
        ))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(
            scan.azimuth_min,
        ))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(
            scan.azimuth_max,
        ))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(
            scan.depth_min,
        ))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(
            scan.depth_max,
        ))),
        Arc::new(Int32Array::from_iter_values(std::iter::once(scan.num_rows))),
        Arc::new(Int32Array::from_iter_values(std::iter::once(scan.num_cols))),
        Arc::new(Float32Array::from_iter_values(std::iter::once(
            scan.rotation_rate,
        ))),
    ];

    RecordBatch::try_new(schema(), columns)
}

pub fn from_struct_array(
    array: &StructArray,
) -> Result<LidarFlatScanOwned, arrow::error::ArrowError> {
    if array.is_empty() {
        return Err(arrow::error::ArrowError::InvalidArgumentError(
            "lidar_flatscan struct array is empty".into(),
        ));
    }
    Ok(LidarFlatScanOwned {
        depths: list_f32(array, 0, "depths")?,
        intensities: list_u8(array, 1, "intensities")?,
        horizontal_fov: scalar_f32(array, 2, "horizontal_fov")?,
        horizontal_resolution: scalar_f32(array, 3, "horizontal_resolution")?,
        azimuth_min: scalar_f32(array, 4, "azimuth_min")?,
        azimuth_max: scalar_f32(array, 5, "azimuth_max")?,
        depth_min: scalar_f32(array, 6, "depth_min")?,
        depth_max: scalar_f32(array, 7, "depth_max")?,
        num_rows: scalar_i32(array, 8, "num_rows")?,
        num_cols: scalar_i32(array, 9, "num_cols")?,
        rotation_rate: scalar_f32(array, 10, "rotation_rate")?,
    })
}

fn list_f32(
    array: &StructArray,
    idx: usize,
    name: &str,
) -> Result<Vec<f32>, arrow::error::ArrowError> {
    let list = array
        .column(idx)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError(format!("flatscan '{name}' not ListArray"))
        })?;
    let values = list
        .values()
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError(format!("flatscan '{name}' inner not Float32"))
        })?;
    Ok(values.values().to_vec())
}

fn list_u8(
    array: &StructArray,
    idx: usize,
    name: &str,
) -> Result<Vec<u8>, arrow::error::ArrowError> {
    let list = array
        .column(idx)
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError(format!("flatscan '{name}' not ListArray"))
        })?;
    let values = list
        .values()
        .as_any()
        .downcast_ref::<UInt8Array>()
        .ok_or_else(|| {
            arrow::error::ArrowError::SchemaError(format!("flatscan '{name}' inner not UInt8"))
        })?;
    Ok(values.values().to_vec())
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
            arrow::error::ArrowError::SchemaError(format!("flatscan '{name}' not Float32"))
        })
        .map(|a| a.value(0))
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
            arrow::error::ArrowError::SchemaError(format!("flatscan '{name}' not Int32"))
        })
        .map(|a| a.value(0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_record_batch() {
        let depths = [0.5_f32, 1.2, 2.7, 3.0, 1.8, 0.9, 4.5, 2.1];
        let intensities = [10_u8, 50, 200, 100, 75, 25, 220, 180];
        let scan = LidarFlatScan {
            depths: &depths,
            intensities: &intensities,
            horizontal_fov: 270.0,
            horizontal_resolution: 0.25,
            azimuth_min: -135.0,
            azimuth_max: 135.0,
            depth_min: 0.1,
            depth_max: 30.0,
            num_rows: 1,
            num_cols: 8,
            rotation_rate: 10.0,
        };

        let batch = to_record_batch(&scan).expect("convert");

        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 11);

        let depths_col = batch
            .column(0)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("depths is ListArray");
        let depths_inner = depths_col
            .values()
            .as_any()
            .downcast_ref::<Float32Array>()
            .expect("depths inner is Float32Array");
        assert_eq!(depths_inner.len(), 8);
        assert_eq!(depths_inner.value(0), 0.5);
        assert_eq!(depths_inner.value(7), 2.1);

        let intensities_col = batch
            .column(1)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("intensities is ListArray");
        let intensities_inner = intensities_col
            .values()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .expect("intensities inner is UInt8Array");
        assert_eq!(intensities_inner.len(), 8);
        assert_eq!(intensities_inner.value(0), 10);
        assert_eq!(intensities_inner.value(7), 180);

        let fov_col = batch
            .column(2)
            .as_any()
            .downcast_ref::<Float32Array>()
            .expect("horizontal_fov is Float32Array");
        assert_eq!(fov_col.value(0), 270.0);

        let cols_col = batch
            .column(9)
            .as_any()
            .downcast_ref::<Int32Array>()
            .expect("num_cols is Int32Array");
        assert_eq!(cols_col.value(0), 8);
    }

    #[test]
    fn from_struct_array_round_trips() {
        let depths = [0.5_f32, 1.2, 2.7, 3.0];
        let intensities = [10_u8, 50, 200, 100];
        let scan = LidarFlatScan {
            depths: &depths,
            intensities: &intensities,
            horizontal_fov: 270.0,
            horizontal_resolution: 0.25,
            azimuth_min: -135.0,
            azimuth_max: 135.0,
            depth_min: 0.1,
            depth_max: 30.0,
            num_rows: 1,
            num_cols: 4,
            rotation_rate: 10.0,
        };
        let batch = to_record_batch(&scan).expect("to");
        let array = StructArray::from(batch);
        let owned = from_struct_array(&array).expect("from");
        assert_eq!(owned.depths, depths);
        assert_eq!(owned.intensities, intensities);
        assert_eq!(owned.horizontal_fov, 270.0);
        assert_eq!(owned.num_cols, 4);
        assert_eq!(owned.rotation_rate, 10.0);
    }
}

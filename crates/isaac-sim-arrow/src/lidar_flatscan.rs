use std::sync::Arc;

use arrow::array::{ArrayRef, Float32Array, Int32Array, ListArray, UInt8Array};
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

pub fn schema() -> SchemaRef {
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
        Arc::new(Float32Array::from(vec![scan.horizontal_fov])),
        Arc::new(Float32Array::from(vec![scan.horizontal_resolution])),
        Arc::new(Float32Array::from(vec![scan.azimuth_min])),
        Arc::new(Float32Array::from(vec![scan.azimuth_max])),
        Arc::new(Float32Array::from(vec![scan.depth_min])),
        Arc::new(Float32Array::from(vec![scan.depth_max])),
        Arc::new(Int32Array::from(vec![scan.num_rows])),
        Arc::new(Int32Array::from(vec![scan.num_cols])),
        Arc::new(Float32Array::from(vec![scan.rotation_rate])),
    ];

    RecordBatch::try_new(schema(), columns)
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
}

use std::sync::Arc;

use arrow::array::{ArrayRef, Float32Array, Int32Array, ListArray};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

pub struct LidarPointCloud<'a> {
    pub azimuth: &'a [f32],
    pub elevation: &'a [f32],
    pub distance: &'a [f32],
    pub intensity: &'a [f32],
    pub num_points: i32,
}

pub fn schema() -> SchemaRef {
    let f32_list = |name: &str| {
        Field::new(
            name,
            DataType::List(Arc::new(Field::new("item", DataType::Float32, false))),
            false,
        )
    };
    Arc::new(Schema::new(vec![
        f32_list("azimuth"),
        f32_list("elevation"),
        f32_list("distance"),
        f32_list("intensity"),
        Field::new("num_points", DataType::Int32, false),
    ]))
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
        Arc::new(list_f32(pc.azimuth)),
        Arc::new(list_f32(pc.elevation)),
        Arc::new(list_f32(pc.distance)),
        Arc::new(list_f32(pc.intensity)),
        Arc::new(Int32Array::from(vec![pc.num_points])),
    ];
    RecordBatch::try_new(schema(), columns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_record_batch() {
        let azimuth = [0.0_f32, 1.0, 2.0];
        let elevation = [0.1_f32, 0.2, 0.3];
        let distance = [5.0_f32, 6.0, 7.0];
        let intensity = [0.1_f32, 0.5, 0.9];
        let pc = LidarPointCloud {
            azimuth: &azimuth,
            elevation: &elevation,
            distance: &distance,
            intensity: &intensity,
            num_points: 3,
        };
        let batch = to_record_batch(&pc).expect("convert");
        assert_eq!(batch.num_rows(), 1);
        assert_eq!(batch.num_columns(), 5);

        let dist_col = batch
            .column(2)
            .as_any()
            .downcast_ref::<ListArray>()
            .expect("distance is ListArray");
        let inner = dist_col
            .values()
            .as_any()
            .downcast_ref::<Float32Array>()
            .expect("inner is Float32Array");
        assert_eq!(inner.len(), 3);
        assert_eq!(inner.value(2), 7.0);
    }
}

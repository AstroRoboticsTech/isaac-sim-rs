use std::sync::{Arc, OnceLock};

use arrow::array::{ArrayRef, Float32Array, Int32Array, ListArray};
use arrow::buffer::OffsetBuffer;
use arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use arrow::record_batch::RecordBatch;

pub struct LidarPointCloud<'a> {
    pub points: &'a [f32],
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
        Arc::new(Int32Array::from(vec![pc.num_points])),
        Arc::new(Int32Array::from(vec![pc.width])),
        Arc::new(Int32Array::from(vec![pc.height])),
    ];
    RecordBatch::try_new(schema(), columns)
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
}

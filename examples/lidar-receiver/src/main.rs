use arrow::array::{Array, Float32Array, ListArray, StructArray};
use dora_node_api::{ArrowData, DoraNode, Event};

fn main() -> eyre::Result<()> {
    let (_node, mut events) = DoraNode::init_from_env()?;

    while let Some(event) = events.recv() {
        match event {
            Event::Input { id, data, .. } => {
                let summary =
                    summarize_scan(&data).unwrap_or_else(|e| format!("(decode error: {e})"));
                println!("[receiver] {id}: {summary}");
            }
            Event::Stop(_) => {
                println!("[receiver] stop event; exiting");
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

fn summarize_scan(data: &ArrowData) -> eyre::Result<String> {
    let s = data
        .as_any()
        .downcast_ref::<StructArray>()
        .ok_or_else(|| eyre::eyre!("expected StructArray, got {:?}", data.data_type()))?;

    let depths_col = s
        .column_by_name("depths")
        .ok_or_else(|| eyre::eyre!("missing 'depths' column"))?
        .as_any()
        .downcast_ref::<ListArray>()
        .ok_or_else(|| eyre::eyre!("'depths' is not a ListArray"))?;
    let depths = depths_col
        .values()
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| eyre::eyre!("'depths' values are not Float32"))?;

    let fov = scalar_f32(s, "horizontal_fov")?;
    let rate = scalar_f32(s, "rotation_rate")?;
    let n = depths.len();
    let dmin = (0..n)
        .map(|i| depths.value(i))
        .fold(f32::INFINITY, f32::min);
    let dmax = (0..n)
        .map(|i| depths.value(i))
        .fold(f32::NEG_INFINITY, f32::max);

    Ok(format!(
        "n={n} fov={fov:.1}° rate={rate:.1}Hz depth=[{dmin:.3},{dmax:.3}]m"
    ))
}

fn scalar_f32(s: &StructArray, name: &str) -> eyre::Result<f32> {
    let arr = s
        .column_by_name(name)
        .ok_or_else(|| eyre::eyre!("missing '{name}' column"))?
        .as_any()
        .downcast_ref::<Float32Array>()
        .ok_or_else(|| eyre::eyre!("'{name}' is not Float32"))?;
    Ok(arr.value(0))
}

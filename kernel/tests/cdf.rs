use std::{error, sync::Arc};

use arrow::compute::filter_record_batch;
use arrow_array::RecordBatch;
use delta_kernel::engine::sync::SyncEngine;
use itertools::Itertools;

use delta_kernel::engine::arrow_data::ArrowEngineData;
use delta_kernel::{DeltaResult, Table, Version};

mod common;
use common::{load_test_data, to_arrow};

fn read_cdf_for_table(
    test_name: impl AsRef<str>,
    start_version: Version,
    end_version: impl Into<Option<Version>>,
) -> DeltaResult<Vec<RecordBatch>> {
    let test_dir = load_test_data("tests/data", test_name.as_ref()).unwrap();
    let test_path = test_dir.path().join(test_name.as_ref());
    let table = Table::try_from_uri(test_path.to_str().expect("table path to string")).unwrap();
    let engine = Arc::new(SyncEngine::new());
    let table_changes = table.table_changes(engine.as_ref(), start_version, end_version)?;

    // Project out the commit timestamp since file modification time may change anytime git clones
    // or switches branches
    let names = table_changes
        .schema()
        .fields()
        .map(|field| field.name())
        .filter(|name| *name != "_commit_timestamp")
        .collect_vec();
    let schema = table_changes.schema().project(&names)?;
    let scan = table_changes
        .into_scan_builder()
        .with_schema(schema)
        .build()?;
    let batches: Vec<RecordBatch> = scan
        .execute(engine)?
        .map(|scan_result| -> DeltaResult<_> {
            let scan_result = scan_result?;
            let mask = scan_result.full_mask();
            let data = scan_result.raw_data?;
            let record_batch = to_arrow(data)?;
            match mask {
                Some(mask) => Ok(filter_record_batch(&record_batch, &mask.into())?),
                None => Ok(record_batch),
            }
        })
        .try_collect()?;
    Ok(batches)
}

#[test]
fn cdf_with_deletion_vector() -> Result<(), Box<dyn error::Error>> {
    let batches = read_cdf_for_table("cdf-table-with-dv", 0, None)?;
    // Each commit performs the following:
    // 0. Insert  0..=9
    // 1. Remove  [0, 9]
    // 2. Restore [0, 9]
    // 3. Remove  [0, 1, 4, 5]
    // 4. Restore [1, 4]
    // 5. Restore [0, 5] and Remove [3]
    // 6. Restore 3
    let mut expected = vec![
        "+-------+--------------+-----------------+",
        "| value | _change_type | _commit_version |",
        "+-------+--------------+-----------------+",
        "| 0     | insert       | 0               |",
        "| 1     | insert       | 0               |",
        "| 2     | insert       | 0               |",
        "| 3     | insert       | 0               |",
        "| 4     | insert       | 0               |",
        "| 5     | insert       | 0               |",
        "| 6     | insert       | 0               |",
        "| 7     | insert       | 0               |",
        "| 8     | insert       | 0               |",
        "| 9     | insert       | 0               |",
        "| 0     | delete       | 1               |",
        "| 9     | delete       | 1               |",
        "| 0     | insert       | 2               |",
        "| 9     | insert       | 2               |",
        "| 0     | delete       | 3               |",
        "| 1     | delete       | 3               |",
        "| 4     | delete       | 3               |",
        "| 5     | delete       | 3               |",
        "| 1     | insert       | 4               |",
        "| 4     | insert       | 4               |",
        "| 3     | delete       | 5               |",
        "| 0     | insert       | 5               |",
        "| 5     | insert       | 5               |",
        "| 3     | insert       | 6               |",
        "+-------+--------------+-----------------+",
    ];
    sort_lines!(expected);
    assert_batches_sorted_eq!(expected, &batches);
    Ok(())
}

#[test]
fn basic_cdf() -> Result<(), Box<dyn error::Error>> {
    let batches = read_cdf_for_table("cdf-table", 0, None)?;
    let mut expected = vec![
        "+----+--------+------------+------------------+-----------------+",
        "| id | name   | birthday   | _change_type     | _commit_version |",
        "+----+--------+------------+------------------+-----------------+",
        "| 1  | Steve  | 2023-12-22 | insert           | 0               |",
        "| 2  | Bob    | 2023-12-23 | insert           | 0               |",
        "| 3  | Dave   | 2023-12-23 | insert           | 0               |",
        "| 4  | Kate   | 2023-12-23 | insert           | 0               |",
        "| 5  | Emily  | 2023-12-24 | insert           | 0               |",
        "| 6  | Carl   | 2023-12-24 | insert           | 0               |",
        "| 7  | Dennis | 2023-12-24 | insert           | 0               |",
        "| 8  | Claire | 2023-12-25 | insert           | 0               |",
        "| 9  | Ada    | 2023-12-25 | insert           | 0               |",
        "| 10 | Borb   | 2023-12-25 | insert           | 0               |",
        "| 3  | Dave   | 2023-12-22 | update_postimage | 1               |",
        "| 3  | Dave   | 2023-12-23 | update_preimage  | 1               |",
        "| 4  | Kate   | 2023-12-22 | update_postimage | 1               |",
        "| 4  | Kate   | 2023-12-23 | update_preimage  | 1               |",
        "| 2  | Bob    | 2023-12-22 | update_postimage | 1               |",
        "| 2  | Bob    | 2023-12-23 | update_preimage  | 1               |",
        "| 7  | Dennis | 2023-12-24 | update_preimage  | 2               |",
        "| 7  | Dennis | 2023-12-29 | update_postimage | 2               |",
        "| 5  | Emily  | 2023-12-24 | update_preimage  | 2               |",
        "| 5  | Emily  | 2023-12-29 | update_postimage | 2               |",
        "| 6  | Carl   | 2023-12-24 | update_preimage  | 2               |",
        "| 6  | Carl   | 2023-12-29 | update_postimage | 2               |",
        "| 7  | Dennis | 2023-12-29 | delete           | 3               |",
        "+----+--------+------------+------------------+-----------------+",
    ];
    sort_lines!(expected);
    assert_batches_sorted_eq!(expected, &batches);
    Ok(())
}

#[test]
fn cdf_non_partitioned() -> Result<(), Box<dyn error::Error>> {
    let batches = read_cdf_for_table("cdf-table-non-partitioned", 0, None)?;
    let mut expected = vec![
             "+----+--------+------------+-------------------+---------------+--------------+----------------+------------------+-----------------+",
             "| id | name   | birthday   | long_field        | boolean_field | double_field | smallint_field | _change_type     | _commit_version |",
             "+----+--------+------------+-------------------+---------------+--------------+----------------+------------------+-----------------+",
             "| 1  | Steve  | 2024-04-14 | 1                 | true          | 3.14         | 1              | insert           | 0               |",
             "| 2  | Bob    | 2024-04-15 | 1                 | true          | 3.14         | 1              | insert           | 0               |",
             "| 3  | Dave   | 2024-04-15 | 2                 | true          | 3.14         | 1              | insert           | 0               |",
             "| 4  | Kate   | 2024-04-15 | 3                 | true          | 3.14         | 1              | insert           | 0               |",
             "| 5  | Emily  | 2024-04-16 | 4                 | true          | 3.14         | 1              | insert           | 0               |",
             "| 6  | Carl   | 2024-04-16 | 5                 | true          | 3.14         | 1              | insert           | 0               |",
             "| 7  | Dennis | 2024-04-16 | 6                 | true          | 3.14         | 1              | insert           | 0               |",
             "| 8  | Claire | 2024-04-17 | 7                 | true          | 3.14         | 1              | insert           | 0               |",
             "| 9  | Ada    | 2024-04-17 | 8                 | true          | 3.14         | 1              | insert           | 0               |",
             "| 10 | Borb   | 2024-04-17 | 99999999999999999 | true          | 3.14         | 1              | insert           | 0               |",
             "| 3  | Dave   | 2024-04-15 | 2                 | true          | 3.14         | 1              | update_preimage  | 1               |",
             "| 3  | Dave   | 2024-04-14 | 2                 | true          | 3.14         | 1              | update_postimage | 1               |",
             "| 4  | Kate   | 2024-04-15 | 3                 | true          | 3.14         | 1              | update_preimage  | 1               |",
             "| 4  | Kate   | 2024-04-14 | 3                 | true          | 3.14         | 1              | update_postimage | 1               |",
             "| 2  | Bob    | 2024-04-15 | 1                 | true          | 3.14         | 1              | update_preimage  | 1               |",
             "| 2  | Bob    | 2024-04-14 | 1                 | true          | 3.14         | 1              | update_postimage | 1               |",
             "| 7  | Dennis | 2024-04-16 | 6                 | true          | 3.14         | 1              | update_preimage  | 2               |",
             "| 7  | Dennis | 2024-04-14 | 6                 | true          | 3.14         | 1              | update_postimage | 2               |",
             "| 5  | Emily  | 2024-04-16 | 4                 | true          | 3.14         | 1              | update_preimage  | 2               |",
             "| 5  | Emily  | 2024-04-14 | 4                 | true          | 3.14         | 1              | update_postimage | 2               |",
             "| 6  | Carl   | 2024-04-16 | 5                 | true          | 3.14         | 1              | update_preimage  | 2               |",
             "| 6  | Carl   | 2024-04-14 | 5                 | true          | 3.14         | 1              | update_postimage | 2               |",
             "| 7  | Dennis | 2024-04-14 | 6                 | true          | 3.14         | 1              | delete           | 3               |",
             "| 1  | Alex   | 2024-04-14 | 1                 | true          | 3.14         | 1              | insert           | 4               |",
             "| 2  | Alan   | 2024-04-15 | 1                 | true          | 3.14         | 1              | insert           | 4               |",
             "+----+--------+------------+-------------------+---------------+--------------+----------------+------------------+-----------------+"
    ];
    sort_lines!(expected);
    assert_batches_sorted_eq!(expected, &batches);
    Ok(())
}

#[test]
fn cdf_with_cdc_and_dvs() -> Result<(), Box<dyn error::Error>> {
    let batches = read_cdf_for_table("cdf-table-with-cdc-and-dvs", 0, None)?;
    let mut expected = vec![
        "+----+--------------------+------------------+-----------------+",
        "| id | comment            | _change_type     | _commit_version |",
        "+----+--------------------+------------------+-----------------+",
        "| 1  | initial            | insert           | 0               |",
        "| 2  | insert1            | insert           | 1               |",
        "| 3  | insert1-delete1    | insert           | 1               |",
        "| 4  | insert1-delete2    | insert           | 1               |",
        "| 5  | insert1-delete2    | insert           | 1               |",
        "| 3  | insert1-delete1    | delete           | 2               |",
        "| 3  | insert1-delete1    | insert           | 4               |",
        "| 4  | insert1-delete2    | delete           | 5               |",
        "| 5  | insert1-delete2    | delete           | 5               |",
        "| 4  | insert1-delete2    | insert           | 7               |",
        "| 5  | insert2            | insert           | 8               |",
        "| 1  | initial            | update_preimage  | 9               |",
        "| 1  | update1            | update_postimage | 9               |",
        "| 2  | insert1            | update_preimage  | 9               |",
        "| 2  | update1            | update_postimage | 9               |",
        "| 3  | insert1-delete1    | update_preimage  | 9               |",
        "| 3  | update1            | update_postimage | 9               |",
        "| 1  | update1            | delete           | 10              |",
        "| 2  | update1            | update_preimage  | 12              |",
        "| 2  | update2            | update_postimage | 12              |",
        "| 6  | insert3            | insert           | 14              |",
        "| 7  | insert3            | insert           | 14              |",
        "| 8  | insert4            | insert           | 15              |",
        "| 9  | insert4            | insert           | 15              |",
        "| 8  | insert4            | delete           | 16              |",
        "| 7  | insert3            | delete           | 16              |",
        "| 10 | merge1-insert      | insert           | 18              |",
        "| 11 | merge1-insert      | insert           | 18              |",
        "| 9  | merge1-update      | update_postimage | 18              |",
        "| 9  | insert4            | update_preimage  | 18              |",
        "| 11 | merge1-insert      | update_preimage  | 20              |",
        "| 11 |                    | update_postimage | 20              |",
        "| 12 | merge2-insert      | insert           | 22              |",
        "| 11 |                    | delete           | 22              |",
        "| 3  | update1            | delete           | 24              |",
        "| 4  | insert1-delete2    | delete           | 24              |",
        "| 5  | insert2            | delete           | 24              |",
        "| 2  | update2            | delete           | 24              |",
        "| 6  | insert3            | delete           | 24              |",
        "| 9  | merge1-update      | delete           | 24              |",
        "| 0  | new                | insert           | 25              |",
        "| 1  | after-large-delete | insert           | 25              |",
        "| 2  |                    | insert           | 25              |",
        "+----+--------------------+------------------+-----------------+",
    ];
    sort_lines!(expected);
    assert_batches_sorted_eq!(expected, &batches);
    Ok(())
}
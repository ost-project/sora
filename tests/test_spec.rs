#![cfg(feature = "index-map")]
#![cfg(feature = "ignore_list")]

use paste::paste;
use serde::Deserialize;
use sora::{ParseResult, Position, SourceMap};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

static TEST_CASES: OnceLock<HashMap<String, TestCase>> = OnceLock::new();

// since Rust doesn't support add tests dynamically...
macro_rules! declare_tests {
    ($($name:ident,)+) => {
        $(
        paste! {
            #[test]
            fn [<test_spec_ $name>]() {
                get_test_case(stringify!($name)).test();
            }
        }
        )+
    };
}

declare_tests! {
    version_valid,
    version_missing,
    version_not_a_number,
    version_numeric_string,
    version_too_high,
    version_too_low,
    sources_missing,
    sources_not_a_list1,
    sources_not_a_list2,
    sources_not_string_or_null,
    sources_and_sources_content_both_null,
    names_missing,
    names_not_a_list1,
    names_not_a_list2,
    names_not_string,
    ignore_list_empty,
    ignore_list_valid1,
    ignore_list_wrong_type1,
    ignore_list_wrong_type2,
    ignore_list_wrong_type3,
    ignore_list_wrong_type4,
    ignore_list_out_of_bounds1,
    ignore_list_out_of_bounds2,
    unrecognized_property,
    invalid_v_l_q_due_to_non_base64_character,
    invalid_v_l_q_due_to_missing_continuation_digits,
    invalid_mapping_not_a_string1,
    invalid_mapping_not_a_string2,
    invalid_mapping_segment_bad_separator,
    invalid_mapping_segment_with_zero_fields,
    invalid_mapping_segment_with_two_fields,
    invalid_mapping_segment_with_three_fields,
    invalid_mapping_segment_with_source_index_out_of_bounds,
    invalid_mapping_segment_with_name_index_out_of_bounds,
    invalid_mapping_segment_with_negative_column,
    invalid_mapping_segment_with_negative_source_index,
    invalid_mapping_segment_with_negative_original_line,
    invalid_mapping_segment_with_negative_original_column,
    invalid_mapping_segment_with_negative_name_index,
    invalid_mapping_segment_with_negative_relative_column,
    invalid_mapping_segment_with_negative_relative_source_index,
    invalid_mapping_segment_with_negative_relative_original_line,
    invalid_mapping_segment_with_negative_relative_original_column,
    invalid_mapping_segment_with_negative_relative_name_index,
    invalid_mapping_segment_with_column_exceeding32_bits,
    invalid_mapping_segment_with_source_index_exceeding32_bits,
    invalid_mapping_segment_with_original_line_exceeding32_bits,
    invalid_mapping_segment_with_original_column_exceeding32_bits,
    invalid_mapping_segment_with_name_index_exceeding32_bits,
    valid_mapping_fields_with32_bit_max_values,
    valid_mapping_large_v_l_q,
    valid_mapping_empty_groups,
    index_map_wrong_type_sections,
    index_map_wrong_type_offset,
    index_map_wrong_type_map,
    index_map_invalid_base_mappings,
    index_map_invalid_overlap,
    index_map_invalid_order,
    index_map_missing_map,
    index_map_missing_offset,
    index_map_missing_offset_line,
    index_map_missing_offset_column,
    index_map_offset_line_wrong_type,
    index_map_offset_column_wrong_type,
    basic_mapping,
    basic_mapping_with_index_map,
    index_map_with_two_concatenated_sources,
    sources_null_sources_content_non_null,
    sources_non_null_sources_content_null,
    transitive_mapping,
    transitive_mapping_with_three_steps,
}

fn get_test_case(name: &str) -> &'static TestCase {
    let tests = TEST_CASES.get_or_init(|| {
        let description = fs::read("tests/source-map-tests/source-map-spec-tests.json").unwrap();
        let description = serde_json::from_slice::<TestDescription>(&description).unwrap();
        description
            .tests
            .into_iter()
            .map(|t| (camel_to_snake(&t.name), t))
            .collect::<HashMap<_, _>>()
    });
    tests.get(name).unwrap()
}

fn camel_to_snake(input: &str) -> String {
    let mut output = String::new();

    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                output.push('_');
            }
            output.push(c.to_lowercase().next().unwrap());
        } else {
            output.push(c);
        }
    }

    output
}

#[derive(Deserialize)]
#[serde(tag = "actionType")]
#[serde(rename_all = "camelCase")]
#[serde(rename_all_fields = "camelCase")]
enum TestAction {
    CheckMapping {
        generated_line: u32,
        generated_column: u32,
        original_source: Option<String>,
        original_line: u32,
        original_column: u32,
        mapped_name: Option<String>,
    },
    CheckIgnoreList {
        present: Vec<String>,
    },
    // ignore at now
    CheckMappingTransitive,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestCase {
    name: String,
    description: String,
    source_map_file: String,
    source_map_is_valid: bool,
    test_actions: Option<Vec<TestAction>>,
}

impl TestCase {
    fn read_map(&self) -> ParseResult<SourceMap> {
        let buf =
            fs::read(Path::new("tests/source-map-tests/resources").join(&self.source_map_file))
                .unwrap();
        SourceMap::from(buf)
    }

    fn test(&self) {
        let msg = format!("[{}] {}", self.name, self.description);

        let parse_result = self.read_map();

        if !self.source_map_is_valid {
            assert!(parse_result.is_err(), "{}", msg);
            return;
        }

        let map = parse_result.expect(&self.description);

        if let Some(actions) = &self.test_actions {
            for action in actions {
                match action {
                    TestAction::CheckMapping {
                        generated_line,
                        generated_column,
                        original_source,
                        original_line,
                        original_column,
                        mapped_name,
                    } => {
                        let mapping = map
                            .find_mapping((*generated_line, *generated_column))
                            .expect(&msg);
                        let actual_source = mapping.source_info().expect(&msg);
                        let actual_source_name = map.sources()[actual_source.id as usize]
                            .as_deref()
                            .map(str::to_owned);
                        assert!(original_source.eq(&actual_source_name), "{}", msg);
                        assert_eq!(
                            Position::from((*original_line, *original_column)),
                            actual_source.position,
                            "{}",
                            msg
                        );
                        if let Some(name) = mapped_name {
                            let actual_name = mapping.name_info().expect(&msg);
                            let actual_name = map.names()[actual_name as usize].as_ref();

                            assert_eq!(name, actual_name, "{}", msg);
                        }
                    }
                    TestAction::CheckIgnoreList { present } => {
                        assert_eq!(present.len(), map.ignore_list().len(), "{}", msg);

                        for (idx, &source_id) in map.ignore_list().iter().enumerate() {
                            let actual_source =
                                map.sources()[source_id as usize].as_deref().expect(&msg);
                            assert_eq!(present[idx], actual_source, "{}", msg);
                        }
                    }
                    TestAction::CheckMappingTransitive => {}
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct TestDescription {
    tests: Vec<TestCase>,
}

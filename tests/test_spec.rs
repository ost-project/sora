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
macro_rules! test {
    ($name:ident) => {
        paste! {
            fn $name(){
                get_test_case(stringify!($name)).test();
            }
            #[test]
            fn [<test_spec_ $name>]() {
                match std::env::var("SPEC_TEST") {
                    Ok(t) => {
                        if t == stringify!($name) {
                            $name()
                        }
                    }
                    Err(..) => $name()
                }
            }
        }
    };
}
macro_rules! ignore {
    ($name:ident) => {};
}

test!(version_valid);
test!(version_missing);
test!(version_not_a_number);
test!(version_numeric_string);
test!(version_too_high);
test!(version_too_low);
// Ignore Note: sora allows empty sources
ignore!(sources_missing);
test!(sources_not_a_list1);
test!(sources_not_a_list2);
test!(sources_not_string_or_null);
test!(sources_and_sources_content_both_null);
// Ignore Note: sora allows empty names
ignore!(names_missing);
test!(names_not_a_list1);
test!(names_not_a_list2);
test!(names_not_string);
test!(ignore_list_empty);
test!(ignore_list_valid1);
test!(ignore_list_wrong_type1);
test!(ignore_list_wrong_type2);
test!(ignore_list_wrong_type3);
test!(ignore_list_wrong_type4);
// Ignore Note: sora allows the ignore list to go out of range
ignore!(ignore_list_out_of_bounds1);
ignore!(ignore_list_out_of_bounds2);
test!(unrecognized_property);
test!(invalid_v_l_q_due_to_non_base64_character);
test!(invalid_v_l_q_due_to_missing_continuation_digits);
test!(invalid_mapping_not_a_string1);
test!(invalid_mapping_not_a_string2);
test!(invalid_mapping_segment_bad_separator);
// Ignore Note: sora allows empty segment
ignore!(invalid_mapping_segment_with_zero_fields);
test!(invalid_mapping_segment_with_two_fields);
test!(invalid_mapping_segment_with_three_fields);
test!(invalid_mapping_segment_with_source_index_out_of_bounds);
test!(invalid_mapping_segment_with_name_index_out_of_bounds);
test!(invalid_mapping_segment_with_negative_column);
test!(invalid_mapping_segment_with_negative_source_index);
// FIXME: reject negative original pos
ignore!(invalid_mapping_segment_with_negative_original_line);
ignore!(invalid_mapping_segment_with_negative_original_column);
test!(invalid_mapping_segment_with_negative_name_index);
test!(invalid_mapping_segment_with_negative_relative_column);
test!(invalid_mapping_segment_with_negative_relative_source_index);
// FIXME: reject negative original pos
ignore!(invalid_mapping_segment_with_negative_relative_original_line);
ignore!(invalid_mapping_segment_with_negative_relative_original_column);
test!(invalid_mapping_segment_with_negative_relative_name_index);
// FIXME: reject pos > u32::MAX
ignore!(invalid_mapping_segment_with_column_exceeding32_bits);
test!(invalid_mapping_segment_with_source_index_exceeding32_bits);
ignore!(invalid_mapping_segment_with_original_line_exceeding32_bits);
ignore!(invalid_mapping_segment_with_original_column_exceeding32_bits);
test!(invalid_mapping_segment_with_name_index_exceeding32_bits);
test!(valid_mapping_fields_with32_bit_max_values);
// FIXME: accept large valid vlq
ignore!(valid_mapping_large_v_l_q);
test!(valid_mapping_empty_groups);
test!(index_map_wrong_type_sections);
test!(index_map_wrong_type_offset);
test!(index_map_wrong_type_map);
// Ignore Note: sora determines whether it is an index map during parsing and doesn't perform strict validation
ignore!(index_map_invalid_base_mappings);
test!(index_map_invalid_overlap);
test!(index_map_invalid_order);
// Ignore Note: sora accepts section without map field
ignore!(index_map_missing_map);
test!(index_map_missing_offset);
test!(index_map_missing_offset_line);
test!(index_map_missing_offset_column);
test!(index_map_offset_line_wrong_type);
test!(index_map_offset_column_wrong_type);
test!(basic_mapping);
test!(basic_mapping_with_index_map);
test!(index_map_with_two_concatenated_sources);
test!(sources_null_sources_content_non_null);
test!(sources_non_null_sources_content_null);
test!(transitive_mapping);
test!(transitive_mapping_with_three_steps);

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

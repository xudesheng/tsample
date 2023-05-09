use std::collections::HashMap;

use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value as JsonValue;
// example: committed=64946176
// example: init=2555904
// example: max=251658240
// example: used=63832000
type TabularItem = (String, JsonValue); // indeed, it's (String, i64)
                                        // example: G1 Survivor Space=committed=20971520, init=0, max=-1, used=20971520
type TabularRow = (String, Vec<TabularItem>); // indeed, it's (String, Vec<(String, i64)>)

pub type TabuleData = Vec<TabularRow>;

lazy_static! {
    static ref TABULAR_DATA_CONTENT: Regex = match Regex::new(r"contents=\{(.*)}\)") {
        Ok(regex) => regex,
        Err(err) => panic!("{}", err),
    };
    static ref KEY_DATA: Regex = match Regex::new(r"contents=\{key=(.*?),(.*?)\}\)\}\)") {
        Ok(regex) => regex,
        Err(err) => panic!("{}", err),
    };
    static ref ITEMNAME_TYPE: Regex = match Regex::new(
        r"itemName=(\w+),itemType=javax.management.openmbean.SimpleType\(name=(.*?)\)\)",
    ) {
        Ok(regex) => regex,
        Err(err) => panic!("{}", err),
    };
    static ref ITEMNAME_VALUE: Regex = match Regex::new(r"(\w*?)=([+-]?([0-9]*[.])?[0-9]+)") {
        Ok(regex) => regex,
        Err(err) => panic!("{}", err),
    };
}

pub fn parse_tabular_data(source: &str) -> anyhow::Result<TabuleData> {
    let mut tabular_data = vec![];
    let content = TABULAR_DATA_CONTENT.captures(source);
    if content.is_none() {
        return Err(anyhow::anyhow!("failed to parse tabular data"));
    }
    let content = content.unwrap();
    let content = match content.get(1) {
        Some(content) => content.as_str(),
        None => return Err(anyhow::anyhow!("failed to parse tabular data")),
    };

    let key_data = KEY_DATA.captures_iter(content);
    for key_data in key_data {
        let key = match key_data.get(1) {
            Some(key) => key.as_str(),
            None => return Err(anyhow::anyhow!("failed to parse tabular data key")),
        };
        let data = match key_data.get(2) {
            Some(data) => data.as_str(),
            None => return Err(anyhow::anyhow!("failed to parse tabular data data")),
        };

        let row = parse_tabular_item_content(data)?;
        tabular_data.push((key.to_string(), row));
    }
    Ok(tabular_data)
}

fn parse_tabular_item_content(items_content: &str) -> anyhow::Result<Vec<TabularItem>> {
    let item_types = parse_tabular_item_type(items_content);
    parse_tabular_item_jsonvalue(item_types, items_content)
}
fn parse_tabular_item_type(items_content: &str) -> HashMap<&str, &str> {
    let mut result = HashMap::new();

    let item_type_iter = ITEMNAME_TYPE.captures_iter(items_content);
    for item_type in item_type_iter {
        let item_name = item_type.get(1);
        let item_type = item_type.get(2);
        if let Some(item_name) = item_name {
            if let Some(item_type) = item_type {
                result.insert(item_name.as_str(), item_type.as_str());
            }
        }
        // match item_name {
        //     Some(item_name) => match item_type {
        //         Some(item_type) => {
        //             result.insert(item_name.as_str(), item_type.as_str());
        //         }
        //         None => {}
        //     },
        //     None => {}
        // }
    }

    result
}
// input: item_type: <committed, java.lang.Long>
// input: committed=95420416, init=0, max=-1, used=95420416
// output: Vec<(String, JsonValue)>
fn parse_tabular_item_jsonvalue(
    item_type: HashMap<&str, &str>,
    tabular_item: &str,
) -> anyhow::Result<Vec<TabularItem>> {
    let itemname_value_iter = ITEMNAME_VALUE.captures_iter(tabular_item);
    let mut result = vec![];
    for itemname_value in itemname_value_iter {
        let itemname = match itemname_value.get(1) {
            Some(itemname) => itemname.as_str(),
            None => return Err(anyhow::anyhow!("itemname is None")),
        };

        let value = match itemname_value.get(2) {
            Some(value) => value.as_str(),
            None => return Err(anyhow::anyhow!("value is None")),
        };
        let item_type = item_type.get(itemname);
        let item_type = match item_type {
            Some(item_type) => item_type,
            None => return Err(anyhow::anyhow!("item_type is None")),
        };
        let item_value = if item_type == &"java.lang.Long" || item_type == &"long" {
            JsonValue::Number(value.parse::<i64>()?.into())
        } else if item_type == &"java.lang.Double" || item_type == &"double" {
            match value.parse::<f64>() {
                Ok(value) => match serde_json::Number::from_f64(value) {
                    Some(value) => JsonValue::Number(value),
                    None => {
                        return Err(anyhow::anyhow!(
                            "Can't convert {} to serde_json::Number",
                            value
                        ))
                    }
                },

                Err(err) => return Err(anyhow::anyhow!("{}", err)),
            }
        } else if item_type == &"java.lang.Float" || item_type == &"float" {
            match value.parse::<f32>() {
                Ok(value) => match serde_json::Number::from_f64(value as f64) {
                    Some(value) => JsonValue::Number(value),
                    None => {
                        return Err(anyhow::anyhow!(
                            "Can't convert {} to serde_json::Number",
                            value
                        ))
                    }
                },
                Err(err) => return Err(anyhow::anyhow!("{}", err)),
            }
        } else if item_type == &"java.lang.Integer" || item_type == &"int" {
            JsonValue::Number(value.parse::<i32>()?.into())
        } else if item_type == &"java.lang.Boolean" || item_type == &"boolean" {
            JsonValue::Bool(value.parse::<bool>()?)
        } else {
            JsonValue::String(value.to_string())
        };

        result.push((itemname.to_string(), item_value));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use regex::Regex;
    use serde_json::Value as JsonValue;
    #[test]
    fn test_tabular_item() {
        let sample = "committed=95420416, init=0, max=-1, used=95420416";
        let mut item_type = HashMap::new();
        item_type.insert("committed", "java.lang.Long");
        item_type.insert("init", "java.lang.Long");
        item_type.insert("max", "java.lang.Long");
        item_type.insert("used", "java.lang.Long");

        let item = parse_tabular_item_jsonvalue(item_type, sample).unwrap();
        let expect = vec![
            (
                "committed".to_string(),
                JsonValue::Number(95420416_i64.into()),
            ),
            ("init".to_string(), JsonValue::Number(0_i64.into())),
            #[allow(clippy::unnecessary_cast)]
            ("max".to_string(), JsonValue::Number((-1 as i64).into())),
            ("used".to_string(), JsonValue::Number(95420416_i64.into())),
        ];

        assert_eq!(item, expect);
    }

    #[test]
    fn test_tabular_item_contents() {
        let sample: &str = r#"value=javax.management.openmbean.CompositeDataSupport(compositeType=javax.management.openmbean.CompositeType(name=java.lang.management.MemoryUsage,items=((itemName=committed,itemType=javax.management.openmbean.SimpleType(name=java.lang.Long)),(itemName=init,itemType=javax.management.openmbean.SimpleType(name=java.lang.Long)),(itemName=max,itemType=javax.management.openmbean.SimpleType(name=java.lang.Long)),(itemName=used,itemType=javax.management.openmbean.SimpleType(name=java.lang.Long)))),contents={committed=95420416, init=0, max=-1, used=95420416"#;
        let result = parse_tabular_item_content(sample);
        assert!(result.is_ok());
        let result = result.unwrap();
        let expect = vec![
            (
                "committed".to_string(),
                JsonValue::Number(95420416_i64.into()),
            ),
            ("init".to_string(), JsonValue::Number(0_i64.into())),
            #[allow(clippy::unnecessary_cast)]
            ("max".to_string(), JsonValue::Number((-1 as i64).into())),
            ("used".to_string(), JsonValue::Number(95420416_i64.into())),
        ];
        assert_eq!(result, expect);
    }

    #[test]
    fn test_tabular_data_aftergc() {
        let sample_file = "sampledata/LastGcInfo_memoryUsageAfterGc.txt";
        let sample_data = std::fs::read_to_string(sample_file).unwrap();
        assert!(sample_data.len() > 100);
        let result = parse_tabular_data(&sample_data);
        assert!(result.is_ok());
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 6);

        let mut keys = HashSet::new();
        for item in result {
            let item_name = item.0;
            let item_type = item.1;
            assert_eq!(item_type.len(), 4);
            keys.insert(item_name);
        }
        let mut sample_keys = HashSet::new();
        sample_keys.insert("G1 Survivor Space".to_owned());
        sample_keys.insert("Compressed Class Space".to_owned());
        sample_keys.insert("Metaspace".to_owned());
        sample_keys.insert("Code Cache".to_owned());
        sample_keys.insert("G1 Old Gen".to_owned());
        sample_keys.insert("G1 Eden Space".to_owned());

        assert_eq!(keys, sample_keys);
    }

    #[test]
    fn test_tabular_data_beforegc() {
        let sample_file = "sampledata/LastGcInfo_memoryUsageBeforeGc.txt";
        let sample_data = std::fs::read_to_string(sample_file).unwrap();
        assert!(sample_data.len() > 100);
        let result = parse_tabular_data(&sample_data);
        assert!(result.is_ok());
        let result = result.unwrap();
        println!("{:?}", result);
        assert_eq!(result.len(), 6);

        let mut keys = HashSet::new();
        for item in result {
            let item_name = item.0;
            let item_type = item.1;
            assert_eq!(item_type.len(), 4);
            keys.insert(item_name);
        }
        let mut sample_keys = HashSet::new();
        sample_keys.insert("G1 Survivor Space".to_owned());
        sample_keys.insert("Compressed Class Space".to_owned());
        sample_keys.insert("Metaspace".to_owned());
        sample_keys.insert("Code Cache".to_owned());
        sample_keys.insert("G1 Old Gen".to_owned());
        sample_keys.insert("G1 Eden Space".to_owned());

        assert_eq!(keys, sample_keys);
    }
    #[test]
    fn test_regex_tabular_data() {
        let sample_file = "sampledata/LastGcInfo_memoryUsageAfterGc.txt";
        let sample_data = std::fs::read_to_string(sample_file).unwrap();
        assert!(sample_data.len() > 100);

        let re = Regex::new(r"contents=\{(.*)}\)").unwrap();
        let caps = re.captures(&sample_data).unwrap();
        let tabular_data = caps.get(1).unwrap().as_str();
        println!("{}", tabular_data);
        assert!(tabular_data.len() > 100);
    }

    #[test]
    fn test_regex_tabular_data_2() {
        let sample_file = "sampledata/LastGcInfo_memoryUsageAfterGc.txt";
        let sample_data = std::fs::read_to_string(sample_file).unwrap();
        assert!(sample_data.len() > 100);

        let re = Regex::new(r"contents=\{(.*)}\)").unwrap();
        let caps = re.captures(&sample_data).unwrap();
        let tabular_data = caps.get(1).unwrap().as_str();
        // println!("{}", tabular_data);
        assert!(tabular_data.len() > 100);

        let re = Regex::new(r"contents=\{key=(.*?),(.*?)\}").unwrap();
        let matches = re.find_iter(tabular_data).collect::<Vec<_>>();
        for cap in matches.iter() {
            println!("{}", cap.as_str());
        }
    }

    #[test]
    fn test_regex_tabular_data_3() {
        let sample_file = "sampledata/LastGcInfo_memoryUsageAfterGc.txt";
        let sample_data = std::fs::read_to_string(sample_file).unwrap();
        assert!(sample_data.len() > 100);

        let re = Regex::new(r"contents=\{(.*)}\)").unwrap();
        let caps = re.captures(&sample_data).unwrap();
        let tabular_data = caps.get(1).unwrap().as_str();
        // println!("{}", tabular_data);
        assert!(tabular_data.len() > 100);

        let re = Regex::new(r"contents=\{key=(.*?),(.*?)\}\)\}\)").unwrap();
        let matches = re.captures_iter(tabular_data).collect::<Vec<_>>();
        for cap in matches.iter() {
            println!(
                "{}->{}",
                cap.get(1).unwrap().as_str(),
                cap.get(2).unwrap().as_str()
            );
        }
    }

    #[test]
    fn test_regex_tabular_data_4() {
        let sample_file = "sampledata/LastGcInfo_memoryUsageAfterGc.txt";
        let sample_data = std::fs::read_to_string(sample_file).unwrap();
        assert!(sample_data.len() > 100);

        let re = Regex::new(r"contents=\{(.*)}\)").unwrap();
        let caps = re.captures(&sample_data).unwrap();
        let tabular_data = caps.get(1).unwrap().as_str();
        // println!("{}", tabular_data);
        assert!(tabular_data.len() > 100);

        let re = Regex::new(r"contents=\{key=(.*?),(.*?)\}\)\}\)").unwrap();
        let re_value = Regex::new(
            r"itemName=(\w+),itemType=javax.management.openmbean.SimpleType\(name=(.*?)\)\)",
        )
        .unwrap();
        let matches = re.captures_iter(tabular_data).collect::<Vec<_>>();
        for cap in matches.iter() {
            let value = cap.get(2).unwrap().as_str();
            let item_headers = re_value.captures_iter(value).collect::<Vec<_>>();
            for item_header in item_headers.iter() {
                println!(
                    "{}:\t{}->{}",
                    cap.get(1).unwrap().as_str(),
                    item_header.get(1).unwrap().as_str(),
                    item_header.get(2).unwrap().as_str()
                );
            }
        }
    }

    #[test]
    fn test_regex_tabular_data_5() {
        let sample_file = "sampledata/LastGcInfo_memoryUsageAfterGc.txt";
        let sample_data = std::fs::read_to_string(sample_file).unwrap();
        assert!(sample_data.len() > 100);

        let re = Regex::new(r"contents=\{(.*)}\)").unwrap();
        let caps = re.captures(&sample_data).unwrap();
        let tabular_data = caps.get(1).unwrap().as_str();
        // println!("{}", tabular_data);
        assert!(tabular_data.len() > 100);

        let re = Regex::new(r"contents=\{key=(.*?),(.*?)\}\)\}\)").unwrap();
        let re_value_content = Regex::new(r"contents=\{(.*)").unwrap();
        let re_value_item = Regex::new(r"(\w*?)=([+-]?([0-9]*[.])?[0-9]+)").unwrap();
        let matches = re.captures_iter(tabular_data).collect::<Vec<_>>();
        for cap in matches.iter() {
            let value = cap.get(2).unwrap().as_str();
            let item_content = re_value_content.captures(value).unwrap();
            let value_items = re_value_item
                .captures_iter(item_content.get(1).unwrap().as_str())
                .collect::<Vec<_>>();
            for value_item in value_items.iter() {
                println!(
                    "{}:\t{}->\t{}:{}",
                    cap.get(1).unwrap().as_str(),
                    item_content.get(1).unwrap().as_str(),
                    value_item.get(1).unwrap().as_str(),
                    value_item.get(2).unwrap().as_str()
                );
            }
        }
    }
}

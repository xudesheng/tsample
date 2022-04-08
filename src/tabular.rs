use serde_json::Value as JsonValue;

// example: committed=64946176
// example: init=2555904
// example: max=251658240
// example: used=63832000
type TabularItem = (String, JsonValue); // indeed, it's (String, i64)
                                        // example: G1 Survivor Space=committed=20971520, init=0, max=-1, used=20971520
type TabularRow = (String, Vec<TabularItem>); // indeed, it's (String, Vec<(String, i64)>)

type TabuleData = Vec<TabularRow>;

pub fn parse_tabular_data(tabular_data: &str) -> TabuleData {
    let mut tabular_data = tabular_data.split("\n");
    let mut rows = Vec::new();
    while let Some(row) = tabular_data.next() {
        let mut row = row.split(",");
        let mut items = Vec::new();
        while let Some(item) = row.next() {
            let mut item = item.split("=");
            let key = item.next().unwrap();
            let value = item.next().unwrap();
            items.push((key.to_string(), JsonValue::String(value.to_string())));
        }
        rows.push((row.next().unwrap().to_string(), items));
    }
    rows
}

pub fn parse_tabular_row(tabular_row: &str) -> TabularRow {
    let mut tabular_row = tabular_row.split(",");
    let name = tabular_row.next().unwrap();
    let mut items = Vec::new();
    while let Some(item) = tabular_row.next() {
        let mut item = item.split("=");
        let key = item.next().unwrap();
        let value = item.next().unwrap();
        items.push((key.to_string(), JsonValue::String(value.to_string())));
    }
    (name.to_string(), items)
}

pub fn parse_tabular_item(tabular_item: &str) -> anyhow::Result<TabularItem> {
    let mut tabular_item = tabular_item.split('=');
    let key = match tabular_item.next() {
        Some(key) => key,
        None => return Err(anyhow::anyhow!("empty key")),
    };
    let value = match tabular_item.next() {
        Some(value) => value,
        None => return Err(anyhow::anyhow!("empty value")),
    };

    match value.parse::<i64>() {
        Ok(value) => Ok((key.to_string(), JsonValue::Number(value.into()))),
        Err(_) => Ok((key.to_string(), JsonValue::String(value.to_string()))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use serde_json::Value as JsonValue;
    #[test]
    fn test_tabular_item() {
        let sample = "committed=64946176";
        let item = parse_tabular_item(sample).unwrap();
        let expect = (
            "committed".to_owned(),
            JsonValue::Number(64946176_i64.into()),
        );
        assert_eq!(item, expect);
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

        let re = Regex::new(r"\[(.+?)\]=.*?contents=\{(.*?)\)\}").unwrap();
        let caps = re.captures(tabular_data).unwrap();
        for cap in caps.iter() {
            println!("{}", cap.unwrap().as_str());
        }
    }
}

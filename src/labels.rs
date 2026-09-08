use crate::xkb_parser::XkbParser;

pub struct Labels {
    pub all: Vec<Vec<String>>,
    pub column: Vec<Vec<String>>,
    pub divisions: Vec<Vec<String>>,
}

// Returns labels to be displayed on grid
impl Labels {
    pub fn rebuild(xkb_parser: &XkbParser) -> Self {
        let top_row: Vec<u32> = (16..26).collect();
        let middle_row: Vec<u32> = (30..40).collect();
        let bottom_row: Vec<u32> = (44..54).collect();

        // Main 30 alpha keys
        // let all_keys: Vec<u32> = (16..26).chain(30..40).chain(44..54).collect();
        let all_keys: Vec<u32> = top_row
            .iter()
            .chain(middle_row.iter())
            .chain(bottom_row.iter())
            .copied()
            .collect();

        // Home row keys
        let hr_keys: Vec<u32> = (30..40).collect();

        // All keys for the main grid
        let all: Vec<Vec<String>> = all_keys
            .iter()
            .map(|&line| {
                hr_keys
                    .iter()
                    .map(|&col| {
                        format!(
                            "{}   {}",
                            xkb_parser.key_code_to_char(col),
                            xkb_parser.key_code_to_char(line)
                        )
                        .to_uppercase()
                    })
                    .collect()
            })
            .collect();

        // Only one column
        let column: Vec<Vec<String>> = all_keys
            .iter()
            .map(|&key| vec![xkb_parser.key_code_to_char(key).to_string().to_uppercase()])
            .collect();

        // Subdivisions
        let divisions: Vec<Vec<String>> = [&top_row, &middle_row, &bottom_row]
            .into_iter()
            .map(|line| {
                line.iter()
                    .map(|&col| xkb_parser.key_code_to_char(col).to_string().to_uppercase())
                    .collect()
            })
            .collect();

        Labels {
            all,
            column,
            divisions,
        }
    }
}

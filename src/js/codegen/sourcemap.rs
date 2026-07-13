use crate::span::Pos;

#[derive(Debug, Clone)]
pub struct SourceMap {
    mappings: Vec<Mapping>,
}

#[derive(Debug, Clone)]
pub struct Mapping {
    pub generated: Pos,
    pub original: Pos,
}

impl SourceMap {
    pub fn new() -> Self {
        SourceMap {
            mappings: Vec::new(),
        }
    }

    pub fn add_mapping(&mut self, generated: Pos, original: Pos) {
        self.mappings.push(Mapping {
            generated,
            original,
        });
    }

    pub fn mappings(&self) -> &[Mapping] {
        &self.mappings
    }

    pub fn into_mappings(self) -> Vec<Mapping> {
        self.mappings
    }

    pub fn generate_vlq(&self) -> String {
        let mut result = String::new();
        let _prev_gen_line = 0usize;
        let mut prev_gen_col = 0usize;
        let mut prev_orig_line = 0usize;
        let mut prev_orig_col = 0usize;

        let mut mappings_by_line: Vec<Vec<&Mapping>> = Vec::new();
        for m in &self.mappings {
            let line = m.generated.line;
            while mappings_by_line.len() <= line {
                mappings_by_line.push(Vec::new());
            }
            mappings_by_line[line].push(m);
        }

        for (line_idx, line_mappings) in mappings_by_line.iter().enumerate() {
            if line_idx > 0 {
                result.push(';');
            }
            if line_mappings.is_empty() {
                continue;
            }
            for (i, m) in line_mappings.iter().enumerate() {
                if i > 0 {
                    result.push(',');
                }
                let gen_col = m.generated.column;
                let orig_line = m.original.line;
                let orig_col = m.original.column;

                let diff_gen_col = if i == 0 {
                    gen_col as isize
                } else {
                    gen_col as isize - prev_gen_col as isize
                };
                let diff_orig_line = orig_line as isize - prev_orig_line as isize;
                let diff_orig_col = orig_col as isize - prev_orig_col as isize;

                let vlq = format!(
                    "{}{}{}{}",
                    encode_vlq(diff_gen_col),
                    encode_vlq(diff_orig_line),
                    encode_vlq(diff_orig_col),
                    "A",
                );
                result.push_str(&vlq);

                prev_gen_col = gen_col;
                prev_orig_line = orig_line;
                prev_orig_col = orig_col;
            }
        }

        result
    }
}

impl Default for SourceMap {
    fn default() -> Self {
        SourceMap::new()
    }
}

fn encode_vlq(value: isize) -> String {
    let vlq = if value >= 0 {
        (value as u64) << 1
    } else {
        (((-value) as u64) << 1) | 1
    };

    let mut result = String::new();
    let mut remaining = vlq;
    loop {
        let mut digit = (remaining & 0x1f) as u8;
        remaining >>= 5;
        if remaining > 0 {
            digit |= 0x20;
        }
        result.push(BASE64_CHARS[digit as usize] as char);
        if remaining == 0 {
            break;
        }
    }
    result
}

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

use parking_lot::Mutex;
use pyo3::exceptions;
use pyo3::prelude::*;
use rayon::prelude::*;
use std::cmp::{min, max};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::sync::Arc;

const WAGNER_FISCHER_ARR_INIT: [usize;100] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
    11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
    21, 22, 23, 24, 25, 26, 27, 28, 29, 30,
    31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
    41, 42, 43, 44, 45, 46, 47, 48, 49, 50,
    51, 52, 53, 54, 55, 56, 57, 58, 59, 60,
    61, 62, 63, 64, 65, 66, 67, 68, 69, 70,
    71, 72, 73, 74, 75, 76, 77, 78, 79, 80,
    81, 82, 83, 84, 85, 86, 87, 88, 89, 90,
    91, 92, 93, 94, 95, 96, 97, 98, 99
];
const MBLEVEN_MATRIX: [&[u8];63] = [
    b"re", b"", b"", b"", b"", b"", b"",
    b"de", b"", b"", b"", b"", b"", b"",
    b"rre", b"ide", b"die", b"", b"", b"", b"",
    b"rde", b"dre", b"", b"", b"", b"", b"",
    b"dde", b"", b"", b"", b"", b"", b"",
    b"rrre", b"idre", b"irde", b"ride", b"rdie", b"drie", b"dire",
    b"rrde", b"rdre", b"drre", b"idde", b"dide", b"ddie", b"",
    b"rdde", b"drde", b"ddre", b"", b"", b"", b"",
    b"ddde", b"", b"", b"", b"", b"", b"",
];
const MATRIX_ROW_INDEX: [usize;3] = [0, 2, 5];

#[pyclass]
#[text_signature = "(file_path, separator, /)"]
struct Searcher {
    indices: HashMap<usize, String>,
    max_length: usize,
    separator: String,
}

#[pymethods]
impl Searcher {
    #[new]
    fn new(
        file_path: &str,
        separator: &str,
    ) -> PyResult<Self> {
        let mut indices = HashMap::<usize, String>::new();
        let mut max_length = 0;
        let input_file;
        match File::open(file_path) {
            Ok(file) => {
                input_file = file;
            },
            Err(error) => {
                return Err(
                    exceptions::PyRuntimeError::new_err(error)
                );
            }
        }
        let input_file = BufReader::new(input_file);

        let mut prefix_len;
        for line in input_file.lines() {
            if let Ok(line) = line {
                if separator.is_empty() {
                    prefix_len = line.len();
                } else if let Some(separator_pos) = line.find(separator) {
                    prefix_len = separator_pos;
                } else {
                    prefix_len = line.len();
                }

                if max_length < prefix_len {
                    max_length = prefix_len;
                }
                let index = indices.entry(prefix_len).or_insert(String::new());
                index.push_str(&line);
                index.push('\n');
            }
        }

        Ok(
            Searcher {
                indices,
                max_length,
                separator: separator.to_string()
            }
        )
    }

    #[text_signature = "(pattern, max_distance, /)"]
    fn search(
        &self,
        pattern: &str,
        max_distance: usize,
    ) -> PyResult<Vec<String>> {
        let results = Arc::new(Mutex::new(Vec::new()));
        let pattern_len = pattern.len();
        let from_len = max(pattern_len - max_distance, 0);
        let to_len = min(pattern_len + max_distance, self.max_length);

        let distance_function = match max_distance {
            max_distance if max_distance > 3 => Searcher::wagner_fischer,
            max_distance if max_distance <= 3 => Searcher::mbleven,
            _ => Searcher::wagner_fischer,
        };

        for current_len in from_len..to_len + 1 {
            if let Some(index) = self.indices.get(&current_len) {
                index
                .trim_end()
                .par_split('\n')
                .for_each(
                    |x| {
                        let matched: bool;
                        if self.separator.is_empty() {
                            matched = distance_function(pattern, x, max_distance);
                        } else if let Some(separator_pos) = x.find(&self.separator) {
                            matched = distance_function(pattern, &x[0..separator_pos], max_distance);
                        } else {
                            matched = distance_function(pattern, x, max_distance);
                        }

                        if matched {
                            results.lock().push(x.to_string());
                        }
                    }
                );
            }
        }

        let results_locked = results.lock();

        Ok(results_locked.to_vec())
    }

    #[staticmethod]
    fn mbleven(
        first_string: &str,
        second_string: &str,
        max_distance: usize,
    ) -> bool {
        let mut i: usize;
        let mut j: usize;
        let mut c: usize;

        let longer_str;
        let shorter_str;
        if first_string.len() > second_string.len() {
            longer_str = first_string;
            shorter_str = second_string;
        } else {
            longer_str = second_string;
            shorter_str = first_string;
        }

        let matrix_row_index = if max_distance == 0 {
            0
        } else {
            max_distance - 1
        };

        let row = MATRIX_ROW_INDEX[matrix_row_index] + (longer_str.len() - shorter_str.len());
        for col in 0..7 {
            let model = MBLEVEN_MATRIX[row * 7 + col];
            if model.is_empty() {
                break;
            }

            i = 0;
            j = 0;
            c = 0;

            while i < longer_str.len() && j < shorter_str.len() && c <= max_distance {
                if longer_str.as_bytes()[i] != shorter_str.as_bytes()[j] {
                    match model[c] {
                        b'd' => {
                            i += 1;
                        },
                        b'r' => {
                            i += 1;
                            j += 1;
                        },
                        b'i' => {
                            j += 1;
                        },
                        b'e' => {
                            c = max_distance + 1;
                        },
                        _ => (),
                    }
                    c += 1;
                } else {
                    i += 1;
                    j += 1;
                }
            }

            if c + (longer_str.len() - i) + (shorter_str.len() - j) <= max_distance {
                return true;
            }
        }

        false
    }

    #[staticmethod]
    fn wagner_fischer(
        first_string: &str,
        second_string: &str,
        max_distance: usize,
    ) -> bool {
        let mut arr = WAGNER_FISCHER_ARR_INIT;
        let mut dia: usize;
        let mut tmp: usize;

        for i in 1..first_string.len() + 1 {
            dia = i - 1;
            arr[0] = i;

            for j in 1..second_string.len() + 1 {
                tmp = arr[j];

                if first_string.as_bytes()[i - 1] != second_string.as_bytes()[j - 1] {
                    arr[j] = min(min(arr[j], arr[j - 1]), dia) + 1;
                } else {
                    arr[j] = dia;
                }
                dia = tmp;
            }
        }

        arr[second_string.len()] <= max_distance
    }
}




#[pymodule]
fn fastzy(
    _py: Python,
    m: &PyModule,
) -> PyResult<()> {
    m.add_class::<Searcher>()?;

    Ok(())
}

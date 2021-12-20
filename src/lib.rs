use bstr::io::BufReadExt;
use memchr::memmem::Finder;
use parking_lot::Mutex;
use pyo3::exceptions;
use pyo3::prelude::*;
use rayon::prelude::*;
use std::cmp::{min, max};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
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
const MBLEVEN_MATRIX: [[&[u8]; 4]; 4] = [
    [
        &[0],
        &[0],
        &[0],
        &[0],
    ],
    [
        &[3],
        &[1],
        &[0],
        &[0],
    ],
    [
        &[15,  9,  6],
        &[13,  7],
        &[5],
        &[0],
    ],
    [
        &[63, 39, 45, 57, 54, 30, 27],
        &[61, 55, 31, 37, 25, 22],
        &[53, 29, 23],
        &[21],
    ],
];

#[pyclass]
struct Searcher {
    indices: HashMap<usize, (String, Vec<u32>)>,
    max_length: usize,
}

#[pymethods]
impl Searcher {
    #[new]
    fn new(
        file_path: &str,
        separator: &str,
    ) -> PyResult<Self> {
        let mut indices = HashMap::<usize, (String, Vec<u32>)>::new();
        let mut max_length = 0;

        let input_file = match File::open(file_path) {
            Ok(input_file) => BufReader::new(input_file),
            Err(error) => return Err(exceptions::PyRuntimeError::new_err(error)),
        };

        let separator_finder = Finder::new(separator.as_bytes());
        let mut prefix_len = 0;
        input_file.for_byte_line(
            |line| {
                if separator.is_empty() {
                    prefix_len = bytecount::num_chars(line);
                } else if let Some(separator_pos) = separator_finder.find(line) {
                    prefix_len = bytecount::num_chars(unsafe { line.get_unchecked(..separator_pos) });
                } else {
                    prefix_len = bytecount::num_chars(line);
                }

                if max_length < prefix_len {
                    max_length = prefix_len;
                }
                let (index, start_positions) = indices.entry(prefix_len).or_insert_with(|| (String::new(), Vec::new()));
                if let Ok(line) = simdutf8::basic::from_utf8(line) {
                    start_positions.push(index.len() as u32);
                    index.push_str(line);
                }

                Ok(true)
            }
        )?;

        Ok(
            Searcher {
                indices,
                max_length,
            }
        )
    }

    fn search(
        &self,
        pattern: &str,
        max_distance: usize,
    ) -> PyResult<Vec<String>> {
        let results = Arc::new(Mutex::new(Vec::new()));
        let pattern_len = pattern.len();
        let from_len = max(pattern_len - max_distance, 0);
        let to_len = min(pattern_len + max_distance, self.max_length);

        for current_len in from_len..to_len + 1 {
            if let Some((index, start_positions)) = self.indices.get(&current_len) {
                if max_distance > 3 {
                    start_positions.par_iter().enumerate().for_each(
                        |(start_pos_index, &start_pos)| {
                            let entry = unsafe { index.get_unchecked(start_pos as usize..start_pos as usize + current_len) };
                            if Searcher::wagner_fischer(pattern, entry, max_distance) {
                                let entry = if start_positions.len() == start_pos_index + 1 {
                                    &index[start_pos as usize..]
                                } else {
                                    &index[start_pos as usize..start_positions[start_pos_index + 1] as usize]
                                };
                                results.lock().push(entry.to_string());
                            }
                        }
                    );
                } else if current_len < pattern_len {
                    let changes_matrix = MBLEVEN_MATRIX[max_distance][pattern_len - current_len];
                    start_positions.par_iter().enumerate().for_each(
                        |(start_pos_index, &start_pos)| {
                            let entry = unsafe { index.get_unchecked(start_pos as usize..start_pos as usize + current_len) };
                            if self.fast_mbleven(pattern, entry, changes_matrix, max_distance) {
                                let entry = if start_positions.len() == start_pos_index + 1 {
                                    &index[start_pos as usize..]
                                } else {
                                    &index[start_pos as usize..start_positions[start_pos_index + 1] as usize]
                                };
                                results.lock().push(entry.to_string());
                            }
                        }
                    );
                } else {
                    let changes_matrix = MBLEVEN_MATRIX[max_distance][current_len - pattern_len];
                    start_positions.par_iter().enumerate().for_each(
                        |(start_pos_index, &start_pos)| {
                            let entry = unsafe { index.get_unchecked(start_pos as usize..start_pos as usize + current_len) };
                            if self.fast_mbleven(entry, pattern, changes_matrix, max_distance) {
                                let entry = if start_positions.len() == start_pos_index + 1 {
                                    &index[start_pos as usize..]
                                } else {
                                    &index[start_pos as usize..start_positions[start_pos_index + 1] as usize]
                                };
                                results.lock().push(entry.to_string());
                            }
                        }
                    );
                };
            }
        }

        let results_locked = results.lock();

        Ok(results_locked.to_vec())
    }

    fn fast_mbleven<'a>(
        &self,
        first_string: &'a str,
        second_string: &'a str,
        changes_matrix: &[u8],
        max_distance: usize,
    ) -> bool {
        let mut differences: usize;

        for mut m in changes_matrix.iter().copied() {
            differences = 0;

            let mut first_string_chars = first_string.chars();
            let mut second_string_chars = second_string.chars();
            let mut first_string_current_char = first_string_chars.next();
            let mut second_string_current_char = second_string_chars.next();

            loop {
                match (first_string_current_char, second_string_current_char) {
                    (Some(first_string_char), Some(second_string_char)) => {
                        if first_string_char != second_string_char {
                            if m == 0 {
                                differences += 2;

                                break;
                            }

                            differences += 1;
                            if m & 1 > 0 {
                                first_string_current_char = first_string_chars.next();
                            }
                            if m & 2 > 0 {
                                second_string_current_char = second_string_chars.next();
                            }

                            m >>= 2;
                        } else {
                            first_string_current_char = first_string_chars.next();
                            second_string_current_char = second_string_chars.next();
                        }
                    },
                    (Some(_first_string_char), None) => {
                        differences += first_string_chars.count() + 1;

                        break;
                    },
                    (None, Some(_second_string_char)) => {
                        differences += second_string_chars.count() + 1;

                        break;
                    },
                    (None, None) => {
                        break;
                    },
                }
            }

            if differences <= max_distance {
                return true;
            }
        }

        false
    }

    #[staticmethod]
    fn mbleven<'a>(
        mut first_string: &'a str,
        mut second_string: &'a str,
        max_distance: usize,
    ) -> bool {
        let mut differences: usize;

        if max_distance == 0 {
            return first_string == second_string;
        }

        let mut first_string_len = first_string.chars().count();
        let mut second_string_len = second_string.chars().count();

        if first_string_len < second_string_len {
            std::mem::swap(&mut first_string, &mut second_string);
            std::mem::swap(&mut first_string_len, &mut second_string_len);
        }

        let strings_len_difference = first_string_len - second_string_len;
        if max_distance < strings_len_difference {
            return false;
        }

        let changes_matrix = MBLEVEN_MATRIX[max_distance][strings_len_difference];
        for mut m in changes_matrix.iter().copied() {
            differences = 0;

            let mut first_string_chars = first_string.chars();
            let mut second_string_chars = second_string.chars();
            let mut first_string_current_char = first_string_chars.next();
            let mut second_string_current_char = second_string_chars.next();

            loop {
                match (first_string_current_char, second_string_current_char) {
                    (Some(first_string_char), Some(second_string_char)) => {
                        if first_string_char != second_string_char {
                            if m == 0 {
                                differences += 2;

                                break;
                            }

                            differences += 1;
                            if m & 1 > 0 {
                                first_string_current_char = first_string_chars.next();
                            }
                            if m & 2 > 0 {
                                second_string_current_char = second_string_chars.next();
                            }

                            m >>= 2;
                        } else {
                            first_string_current_char = first_string_chars.next();
                            second_string_current_char = second_string_chars.next();
                        }
                    },
                    (Some(_first_string_char), None) => {
                        differences += first_string_chars.count() + 1;

                        break;
                    },
                    (None, Some(_second_string_char)) => {
                        differences += second_string_chars.count() + 1;

                        break;
                    },
                    (None, None) => {
                        break;
                    },
                }
            }

            if differences <= max_distance {
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

        if max_distance == 0 {
            return first_string == second_string;
        }

        for (i, first_string_current_char) in first_string.chars().enumerate() {
            dia = i;
            arr[0] = i + 1;

            for (j, second_string_current_char) in second_string.chars().enumerate() {
                tmp = arr[j + 1];

                if first_string_current_char != second_string_current_char {
                    arr[j + 1] = min(min(arr[j + 1], arr[j]), dia) + 1;
                } else {
                    arr[j + 1] = dia;
                }

                dia = tmp;
            }
        }

        arr[second_string.chars().count()] <= max_distance
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

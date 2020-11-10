#include <algorithm>
#include <array>
#include <fstream>
#include <future>
#include <vector>

#include "pybind11/pybind11.h"
#include "pybind11/stl.h"


class Searcher {
    public:
    Searcher(
       std::string input_file_path,
       std::string separator
    ) {
        std::ifstream input_file(input_file_path);
        if (!input_file.good()) {
            throw std::runtime_error("Cannot open input file: " + input_file_path);
        }

        this->separator = separator;

        std::string line;
        if (separator == "") {
            this->separated = false;
            while (std::getline(input_file, line)) {
                if (line == "") {
                    continue;
                }

                this->lines[line.size()].append(line + "\n");
            }
        } else {
            this->separated = true;

            while (std::getline(input_file, line)) {
                if (line == "") {
                    continue;
                }

                std::size_t prefix_length = line.find_first_of(separator);
                if (prefix_length == std::string::npos || prefix_length == 0) {
                    continue;
                }
                this->lines[prefix_length].append(line + "\n");
            }
        }
    }

    std::vector<std::string> lookup(
        std::string_view pattern,
        std::uint8_t max_distance
    ) {
        std::vector<std::string> results;
        std::vector<std::future<std::vector<std::string>>> futures;

        for (std::uint8_t i = 1; i <= max_distance; i++) {
            futures.push_back(
                std::async(
                    &Searcher::iterate_distances,
                    this,
                    pattern,
                    max_distance,
                    pattern.size() - i
                )
            );
            futures.push_back(
                std::async(
                    &Searcher::iterate_distances,
                    this,
                    pattern,
                    max_distance,
                    pattern.size() + i
                )
            );
        }
        futures.push_back(
            std::async(
                &Searcher::iterate_distances,
                this,
                pattern,
                max_distance,
                pattern.size()
            )
        );

        for (auto & future : futures) {
            auto result = future.get();
            results.insert(results.end(), result.begin(), result.end());
        }

        return results;
    }

    std::vector<std::string> iterate_distances(
        std::string_view pattern,
        std::uint8_t max_distance,
        std::uint8_t lines_index
    ) {
        std::vector<std::string> results;
        std::string_view lines = this->lines[lines_index];

        std::function<bool(std::string_view)> get_distance;
        if (max_distance > 3) {
            get_distance = [this, pattern, max_distance] (std::string_view str) {
                return this->bounded_wagner_fischer(pattern, str, max_distance);
            };
        } else {
            get_distance = [this, pattern, max_distance] (std::string_view str) {
                return this->mbleven(pattern, str, max_distance);
            };
        }

        std::size_t start_of_line = 0;
        std::size_t end_of_line = 0;
        while (start_of_line < lines.size()) {
            if (this->separated) {
                end_of_line = lines.find(this->separator, end_of_line);
            } else {
                end_of_line = lines.find('\n', end_of_line);
            }
            if (end_of_line == std::string::npos) {
                break;
            }

            std::string_view current_line(
                &lines[start_of_line],
                end_of_line - start_of_line
            );
            if (this->separated) {
                end_of_line = lines.find('\n', end_of_line);
            }

            if (get_distance(current_line)) {
                if (this->separated) {
                    std::string_view current_line(
                        &lines[start_of_line],
                        end_of_line - start_of_line
                    );
                    results.push_back(std::string(current_line));
                } else {
                    results.push_back(std::string(current_line));
                }

            }
            end_of_line += 1;
            start_of_line = end_of_line;
        }


        return results;
    }

    static constexpr const char * mbleven_matrix[] = {
        "r", NULL, NULL, NULL, NULL, NULL, NULL,
        "d", NULL, NULL, NULL, NULL, NULL, NULL,
        "rr", "id", "di", NULL, NULL, NULL, NULL,
        "rd", "dr", NULL, NULL, NULL, NULL, NULL,
        "dd", NULL, NULL, NULL, NULL, NULL, NULL,
        "rrr", "idr", "ird", "rid", "rdi", "dri", "dir",
        "rrd", "rdr", "drr", "idd", "did", "ddi", NULL,
        "rdd", "drd", "ddr", NULL, NULL, NULL, NULL,
        "ddd", NULL, NULL, NULL, NULL, NULL, NULL,
    };
    static constexpr int matrix_row_index[3] = {0, 2, 5};
    static inline bool mbleven(
        std::string_view first_string,
        std::string_view second_string,
        const std::uint8_t max_distance
    ) {
        const char * model;
        uint8_t i;
        uint8_t j;
        uint8_t c;

        if (first_string.length() < second_string.length()) {
            first_string.swap(second_string);
        }

        std::int32_t row = matrix_row_index[max_distance - 1] + (first_string.size() - second_string.size());
        for (std::int32_t col = 0; col < 7; col++) {
            model = mbleven_matrix[row * 7 + col];
            if (model == NULL) {
                break;
            }

            i = 0;
            j = 0;
            c = 0;

            while (i < first_string.size() && j < second_string.size() && c <= max_distance) {
                if (first_string[i] != second_string[j]) {
                    switch (model[c]) {
                        case 'd':
                            i++;
                            break;
                        case 'r':
                            i++;
                            j++;
                            break;
                        case '\0':
                            c = max_distance + 1;
                            break;
                        case 'i':
                            j++;
                            break;
                    }
                    c++;
                } else {
                    i++;
                    j++;
                }
            }

            if (c + (first_string.size() - i) + (second_string.size() - j) <= max_distance) {
                return true;
            }
        }

        return false;
    }

    static constexpr std::array<std::uint32_t, 100> arr_init = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99};
    static inline bool bounded_wagner_fischer(
        std::string_view first_string,
        std::string_view second_string,
        const std::uint8_t max_distance
    ) {
        std::array<std::uint32_t, 100> arr;
        std::uint32_t diag;
        std::uint32_t tmp;

        std::copy_n(std::begin(Searcher::arr_init), second_string.size() + 1, std::begin(arr));

        for (std::uint32_t i = 1; i <= first_string.size(); i++) {
            diag = i - 1;
            arr[0] = i;

            for (std::uint32_t j = 1; j <= second_string.size(); j++) {
                tmp = arr[j];

                if (first_string[i - 1] != second_string[j - 1]) {
                    arr[j] = std::min({arr[j], arr[j - 1], diag}) + 1;
                } else {
                    arr[j] = diag;
                }
                diag = tmp;
            }
        }

        return arr[second_string.size()] <= max_distance;
    }

    std::string separator;
    std::vector<char> file_data;
    std::unordered_map<std::uint32_t, std::string> lines;
    bool separated;
};


PYBIND11_MODULE(fastzy, m) {
    pybind11::class_<Searcher>(m, "Searcher")
        .def(
            pybind11::init<std::string, std::string>(),
            "Searcher object that holds the text from the input file. separator argument is used to separate each line at a specific point to be able to process each line into tokens.",
            pybind11::arg("input_file_path"),
            pybind11::arg("separator")
        )
        .def(
            "lookup",
            &Searcher::lookup,
            "Fuzzy search for a specific pattern",
            pybind11::arg("pattern"),
            pybind11::arg("max_distance")
        );

        m.def(
            "mbleven",
            &Searcher::mbleven,
            "mbleven implementation taking two strings and a max distance and returns whether the distance between the strings is lower or equal to the max distance",
            pybind11::arg("first_string"),
            pybind11::arg("second_string"),
            pybind11::arg("max_distance")
        );
        m.def(
            "bounded_wagner_fischer",
            &Searcher::bounded_wagner_fischer,
            "Wagner-Fischer implementation taking two strings and a max distance and returns whether the distance between the strings is lower or equal to the max distance",
            pybind11::arg("first_string"),
            pybind11::arg("second_string"),
            pybind11::arg("max_distance")
        );
}

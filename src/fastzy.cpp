#include <pybind11/pybind11.h>
#include <pybind11/stl.h>
#include <string>
#include <cstring>
#include <vector>
#include <cstdlib>
#include <fstream>
#include <array>
#include <algorithm>
#include <future>
#include <csignal>


class Searcher {
    public:
    Searcher(
       std::string input_file_path,
       std::string separator
    ) {
        std::ifstream input_file(
            input_file_path,
            std::ifstream::in | std::ifstream::binary
        );
        if (!input_file.good()) {
            throw std::runtime_error("Cannot open input file: " + input_file_path);
        }

        std::string line;

        if (separator == "") {
            this->separated = false;

            while (std::getline(input_file, line)) {
                this->lines[line.size()].first.push_back(line);
            }
        } else {
            this->separated = true;

            while (std::getline(input_file, line)) {
                std::size_t prefix_length = line.find_first_of(separator);
                if (prefix_length == std::string::npos) {
                    continue;
                }
                std::size_t suffix_length = line.size() - prefix_length;
                std::string prefix = line.substr(0, prefix_length);
                std::string suffix = line.substr(prefix_length, suffix_length);
                this->lines[prefix.size()].first.push_back(prefix);
                this->lines[prefix.size()].second.push_back(suffix);
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
        const auto & [prefixes, suffixes] = this->lines[lines_index];

        if (max_distance > 3) {
            for (std::uint32_t i = 0; i < prefixes.size(); i++) {
                if (this->wagner_fischer(prefixes[i], pattern) <= max_distance) {
                    if (this->separated) {
                        results.push_back(prefixes[i] + suffixes[i]);
                    } else {
                        results.push_back(prefixes[i]);
                    }
                }
            }
        } else {
            if (lines_index < pattern.size()) {
                for (std::uint32_t i = 0; i < prefixes.size(); i++) {
                    if (this->mbleven(pattern, prefixes[i], max_distance) <= max_distance) {
                        if (this->separated) {
                            results.push_back(prefixes[i] + suffixes[i]);
                        } else {
                            results.push_back(prefixes[i]);
                        }
                    }
                }
            } else {
                for (std::uint32_t i = 0; i < prefixes.size(); i++) {
                    if (this->mbleven(prefixes[i], pattern, max_distance) <= max_distance) {
                        if (this->separated) {
                            results.push_back(prefixes[i] + suffixes[i]);
                        } else {
                            results.push_back(prefixes[i]);
                        }
                    }
                }
            }
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
    inline std::int32_t mbleven(
        std::string_view s1,
        std::string_view s2,
        const std::uint8_t k
    ) {
        const char * model;
        std::int32_t row, col;
        std::int32_t res = k + 1;
        uint8_t i;
        uint8_t j;
        uint8_t c;

        row = matrix_row_index[k - 1] + (s1.size() - s2.size());
        for (col = 0; col < 7; col++) {
            model = mbleven_matrix[row * 7 + col];
            if (model == NULL) {
                break;
            }

            i = 0;
            j = 0;
            c = 0;

            while (i < s1.size() && j < s2.size() && c <= k) {
                if (s1[i] != s2[j]) {
                    switch (model[c]) {
                        case 'd':
                            i++;
                            break;
                        case 'r':
                            i++;
                            j++;
                            break;
                        case '\0':
                            c = k + 1;
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

            res = std::min(res, (std::int32_t)(c + (s1.size() - i) + (s2.size() - j)));
        }

        return res;
    }

    inline std::int32_t wagner_fischer(
        const std::string_view s1,
        const std::string_view s2
    ) {
        std::array<int, 100> arr;
        std::int32_t dia;

        std::copy_n(std::begin(Searcher::arr_init), s2.size(), std::begin(arr));

        for (std::uint32_t i = 1; i <= s1.size(); i++) {
            char chr = s1[i - 1];
            dia = i - 1;
            arr[0] = i;

            for (std::uint32_t j = 1; j <= s2.size(); j++) {
                std::int32_t tmp = arr[j];

                if (chr != s2[j - 1]) {
                    arr[j] = std::min({arr[j], arr[j - 1], dia}) + 1;
                } else {
                    arr[j] = dia;
                }
                dia = tmp;
            }
        }
        dia = arr[s2.size()];

        return dia;
    }

    static constexpr std::array<int, 100> arr_init = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99};
    std::unordered_map<std::uint32_t, std::pair<std::vector<std::string>, std::vector<std::string>>> lines;
    bool separated;
};


PYBIND11_MODULE(fastzy, m) {
    pybind11::class_<Searcher>(m, "Searcher")
        .def(
            pybind11::init<const std::string &, const std::string &>(),
            "Searcher object that holds the text from the input file. separator argument is used to separate each line at a specific point to be able to process each line into tokens.",
            pybind11::arg("input_file_path"),
            pybind11::arg("separator").none(true)
        )
        .def(
            "lookup",
            &Searcher::lookup,
            "Fuzzy search for a specific pattern",
            pybind11::arg("pattern"),
            pybind11::arg("max_distance")
        );
}

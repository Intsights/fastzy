<p align="center">
    <a href="https://github.com/Intsights/fastzy">
        <img src="https://raw.githubusercontent.com/Intsights/fastzy/master/images/logo.png" alt="Logo">
    </a>
    <h3 align="center">
        Python library for fast fuzzy search over a big file written in Rust
    </h3>
</p>

![license](https://img.shields.io/badge/MIT-License-blue)
![Python](https://img.shields.io/badge/Python-3.7%20%7C%203.8%20%7C%203.9%20%7C%203.10%20%7C%203.11%20%7C%203.12-blue)
![Build](https://github.com/Intsights/fastzy/workflows/Build/badge.svg)
[![PyPi](https://img.shields.io/pypi/v/fastzy.svg)](https://pypi.org/project/fastzy/)

## Table of Contents

- [Table of Contents](#table-of-contents)
- [About The Project](#about-the-project)
  - [Built With](#built-with)
  - [Performance](#performance)
  - [Installation](#installation)
- [Usage](#usage)
- [License](#license)
- [Contact](#contact)


## About The Project

Fastzy is a library written in Rust that can search through a file looking for text based on its distance (Levenshtein). For measuring the Levenshtein distance, the library uses mbleven's algorithm. In situations where the requested distance exceeds 3, where mbleven is slower, Wagner-Fischer is used instead of mbleven. This library loads the whole file into memory, and creates a lightweight index based on the length of the lines. The result is that only potential lines are looked up, opposed to a large number of lines.


### Built With

* [mbleven](https://github.com/fujimotos/mbleven)
* [Pyo3](https://github.com/PyO3/pyo3)


### Performance

| Library | Function | Time |
| ------------- | ------------- | ------------- |
| [polyleven](https://github.com/ztane/python-Levenshtein) | polyleven.levenshtein('text') | 8.48s |
| [fastzy](https://github.com/Intsights/fastzy) | fastzy.search('text) | 0.003s |


### Installation

```sh
pip3 install fastzy
```


## Usage

```python
import fastzy

# open a file and index it in memory
searcher = fastzy.Searcher(
    file_path='input_text_file.txt',
    separator='',
)

# search for the input text 'text' with the distance of 1
searcher.search(
    pattern='text',
    max_distance=1,
)
['test', 'texts', 'next']
```


## License

Distributed under the MIT License. See `LICENSE` for more information.


## Contact

Gal Ben David - gal@intsights.com

Project Link: [https://github.com/Intsights/fastzy](https://github.com/Intsights/fastzy)

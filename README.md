<p align="center">
    <a href="https://github.com/wavenator/fastzy">
        <img src="https://raw.githubusercontent.com/wavenator/fastzy/master/images/logo.png" alt="Logo">
    </a>
    <h3 align="center">
        Python library for fast fuzzy search over a big file leveraging C++ and mbleven algorithm
    </h3>
</p>

![license](https://img.shields.io/badge/MIT-License-blue)
![Python](https://img.shields.io/badge/Python-3.6%20%7C%203.7%20%7C%203.8-blue)
![Build](https://github.com/wavenator/fastzy/workflows/Build/badge.svg)
[![PyPi](https://img.shields.io/pypi/v/fastzy.svg)](https://pypi.org/project/fastzy/)

## Table of Contents

- [Table of Contents](#table-of-contents)
- [About The Project](#about-the-project)
  - [Built With](#built-with)
  - [Performance](#performance)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
- [License](#license)
- [Contact](#contact)


## About The Project

Fastzy is a library written in C++ used for searching over a file for a text based on its distance (levenshtein). The library uses mbleven algorithm for a k-bounded levenshtein distance measurement. When the max distance requested is above 3, where mbleven should be slower, the distance algorithm is replaced with Wagner–Fischer.The library at first, loads the whole file into memory, and created a lightweight index, based on the length of the line. It helps to narrow down the amount of lookups to only potential lines.


### Built With

* [mbleven](https://github.com/fujimotos/mbleven)


### Performance

| Library  | Text Size | Function | Time | #Results | Improvement Factor |
| ------------- | ------------- | ------------- | ------------- | ------------- | ------------- |
| [python-Levenshtein](https://github.com/ztane/python-Levenshtein) | 500mb | Levenshtein.distance('text') | 24.2 s | 1249 | 1.0x |
| [fastzy](https://github.com/wavenator/fastzy) | 500mb | fastzy.lookup('text) | 22.2 ms | 1249 | 1090.0x |


### Prerequisites

In order to compile this package you should have GCC & Python development package installed.
* Fedora
```sh
sudo dnf install python3-devel gcc-c++
```
* Ubuntu 18.04
```sh
sudo apt install python3-dev g++-8
```

### Installation

```sh
pip3 install fastzy
```



## Usage

```python
import fastzy

# open a file and index it in memory
searcher = fastzy.Searcher(
    input_file_path='input_text_file.txt',
    separator='',
)

# lookup for the input text 'text' with the distance of 1
searcher.lookup(
    pattern='text',
    max_distance=1,
)
['test', 'texts', 'next']
```


## License

Distributed under the MIT License. See `LICENSE` for more information.


## Contact

Gal Ben David - wavenator@gmail.com

Project Link: [https://github.com/wavenator/fastzy](https://github.com/wavenator/fastzy)

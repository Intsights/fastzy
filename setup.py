import setuptools
import os
import glob


setuptools.setup(
    name='fastzy',
    version='0.2.0',
    author='Gal Ben David',
    author_email='gal@intsights.com',
    url='https://github.com/Intsights/fastzy',
    project_urls={
        'Source': 'https://github.com/Intsights/fastzy',
    },
    license='MIT',
    description='Python library for fast fuzzy search over a big file leveraging C++ and mbleven algorithm',
    long_description=open('README.md').read(),
    long_description_content_type='text/markdown',
    classifiers=[
        'License :: OSI Approved :: MIT License',
        'Programming Language :: Python :: 3.6',
        'Programming Language :: Python :: 3.7',
        'Programming Language :: Python :: 3.8',
        'Programming Language :: Python :: 3.9',
    ],
    keywords='fuzzy levenshtein mbleven wagner-fischer c++ approximate',
    python_requires='>=3.6',
    zip_safe=False,
    package_data={},
    setup_requires=[
        'pytest-runner',
    ],
    tests_require=[
        'pytest',
    ],
    include_package_data=True,
    ext_modules=[
        setuptools.Extension(
            name='fastzy',
            sources=glob.glob(
                pathname=os.path.join(
                    'src',
                    'fastzy.cpp',
                ),
            ),
            language='c++',
            extra_compile_args=[
                '-std=c++17',
            ],
            extra_link_args=[],
            include_dirs=[
                'src',
            ],
        ),
    ],
)

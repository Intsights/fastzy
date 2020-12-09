import fastzy
import time
import Levenshtein


searcher = fastzy.Searcher(
    file_path='500mb',
    separator='.',
)

start = time.perf_counter()

results = searcher.search(
    pattern='text',
    max_distance=1,
)

end = time.perf_counter()
print(f'fastzy took: {end - start} seconds, found {len(results)}')

start = time.perf_counter()

with open('500mb') as lines_file:
    results = []
    for line in lines_file:
        prefix, postfix = line.split('.')
        if Levenshtein.distance(prefix, 'text') <= 1:
            results.append(line)

end = time.perf_counter()
print(f'Levenshtein took: {end - start} seconds, found {len(results)}')

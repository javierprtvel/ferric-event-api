#!/bin/bash

drill --benchmark ./test/benchmark.yml --quiet --stats | tee ./test/benchmark_results.txt
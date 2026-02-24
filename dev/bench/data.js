window.BENCHMARK_DATA = {
  "lastUpdate": 1771913048174,
  "repoUrl": "https://github.com/atl-lang/atlas",
  "entries": {
    "Atlas Runtime Benchmarks": [
      {
        "commit": {
          "author": {
            "name": "proxy",
            "username": "proxikal",
            "email": "proxikal@gmail.com"
          },
          "committer": {
            "name": "proxy",
            "username": "proxikal",
            "email": "proxikal@gmail.com"
          },
          "id": "b28df29ca464a534d511873e010f598826eb3e7d",
          "message": "ci(bench): parse Criterion output to JSON for customSmallerIsBetter",
          "timestamp": "2026-02-24T03:51:04Z",
          "url": "https://github.com/atl-lang/atlas/commit/b28df29ca464a534d511873e010f598826eb3e7d"
        },
        "date": 1771908215607,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "interp_fibonacci_20",
            "value": 38448000,
            "unit": "ns/iter"
          },
          {
            "name": "interp_arrays/len_10k",
            "value": 6057100,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/fibonacci_20",
            "value": 8291499.999999999,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/nested_loops",
            "value": 3337500,
            "unit": "ns/iter"
          },
          {
            "name": "string_replace_large",
            "value": 61865,
            "unit": "ns/iter"
          },
          {
            "name": "array_map_10k_elements",
            "value": 62885,
            "unit": "ns/iter"
          },
          {
            "name": "array_sort_1000_numbers",
            "value": 59876,
            "unit": "ns/iter"
          },
          {
            "name": "array_indexOf_search",
            "value": 59368,
            "unit": "ns/iter"
          },
          {
            "name": "math_arithmetic_10k_ops",
            "value": 7548100,
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_10kb",
            "value": 67258,
            "unit": "ns/iter"
          },
          {
            "name": "json_nested_access_deep",
            "value": 68212,
            "unit": "ns/iter"
          },
          {
            "name": "file_read_1mb",
            "value": 162750,
            "unit": "ns/iter"
          },
          {
            "name": "file_write_1mb",
            "value": 6459400,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/10",
            "value": 124710,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/50",
            "value": 408950,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/200",
            "value": 1391300,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/50",
            "value": 156290,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/200",
            "value": 433820,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/500",
            "value": 1000500,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/lex",
            "value": 1868.1000000000001,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/parse",
            "value": 3657.5,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/compile",
            "value": 5712.5,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/len_5k",
            "value": 2578400,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/trim_3k",
            "value": 598160,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/split_1k",
            "value": 1217400,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/push_1k",
            "value": 21020,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/pop_500",
            "value": 34371,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/len_5k",
            "value": 2602900,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/index_5k",
            "value": 2968600,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/abs_5k",
            "value": 2719500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/sqrt_2k",
            "value": 1046600,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/min_max_3k",
            "value": 1998100,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/type/type_of_3k",
            "value": 23631,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_add_1000",
            "value": 351600,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_sub_1000",
            "value": 349960,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_mul_1000",
            "value": 318690,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_div_1000",
            "value": 321350,
            "unit": "ns/iter"
          },
          {
            "name": "vm_function_multi_arg",
            "value": 458520,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_count_10000",
            "value": 1918400,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_accumulate_5000",
            "value": 2495300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_nested_100x100",
            "value": 3139600,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_create_20",
            "value": 10770,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_set_index_1000",
            "value": 613180,
            "unit": "ns/iter"
          },
          {
            "name": "vm_comparison_ops_5000",
            "value": 2814700,
            "unit": "ns/iter"
          },
          {
            "name": "vm_equality_check_5000",
            "value": 1339600,
            "unit": "ns/iter"
          },
          {
            "name": "vm_string_concat_100",
            "value": 43329,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/100",
            "value": 43423,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/1000",
            "value": 353860,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/5000",
            "value": 1735900,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/10000",
            "value": 3451200,
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "proxikal@gmail.com",
            "name": "proxy",
            "username": "proxikal"
          },
          "committer": {
            "email": "proxikal@gmail.com",
            "name": "proxy",
            "username": "proxikal"
          },
          "distinct": true,
          "id": "9c00df22d61ef132bc8397fe8a9c24c5689f4e0d",
          "message": "ci(bench): parse Criterion output to JSON for customSmallerIsBetter",
          "timestamp": "2026-02-24T00:12:12-05:00",
          "tree_id": "b6beb90cbffe697bc82dae6317ee1906408b733e",
          "url": "https://github.com/atl-lang/atlas/commit/9c00df22d61ef132bc8397fe8a9c24c5689f4e0d"
        },
        "date": 1771913047731,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "interp_fibonacci_20",
            "value": 38241000,
            "unit": "ns/iter"
          },
          {
            "name": "interp_arrays/len_10k",
            "value": 6060900,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/fibonacci_20",
            "value": 8202999.999999999,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/nested_loops",
            "value": 3343100,
            "unit": "ns/iter"
          },
          {
            "name": "string_replace_large",
            "value": 62691,
            "unit": "ns/iter"
          },
          {
            "name": "array_map_10k_elements",
            "value": 62818,
            "unit": "ns/iter"
          },
          {
            "name": "array_sort_1000_numbers",
            "value": 60363,
            "unit": "ns/iter"
          },
          {
            "name": "array_indexOf_search",
            "value": 59976,
            "unit": "ns/iter"
          },
          {
            "name": "math_arithmetic_10k_ops",
            "value": 7510300,
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_10kb",
            "value": 67611,
            "unit": "ns/iter"
          },
          {
            "name": "json_nested_access_deep",
            "value": 68458,
            "unit": "ns/iter"
          },
          {
            "name": "file_read_1mb",
            "value": 157990,
            "unit": "ns/iter"
          },
          {
            "name": "file_write_1mb",
            "value": 6257500,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/10",
            "value": 123610,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/50",
            "value": 405180,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/200",
            "value": 1377000,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/50",
            "value": 153990,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/200",
            "value": 424790,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/500",
            "value": 976280,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/lex",
            "value": 1911.4,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/parse",
            "value": 3765.7,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/compile",
            "value": 5517.299999999999,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/len_5k",
            "value": 2608400,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/trim_3k",
            "value": 606660,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/split_1k",
            "value": 1207600,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/push_1k",
            "value": 20810,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/pop_500",
            "value": 33564,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/len_5k",
            "value": 2625200,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/index_5k",
            "value": 2984300,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/abs_5k",
            "value": 2681100,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/sqrt_2k",
            "value": 1032100,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/min_max_3k",
            "value": 2030500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/type/type_of_3k",
            "value": 23342,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_add_1000",
            "value": 356020,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_sub_1000",
            "value": 353870,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_mul_1000",
            "value": 323030,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_div_1000",
            "value": 325110,
            "unit": "ns/iter"
          },
          {
            "name": "vm_function_multi_arg",
            "value": 394540,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_count_10000",
            "value": 1912200,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_accumulate_5000",
            "value": 1870400,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_nested_100x100",
            "value": 3157500,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_create_20",
            "value": 10558,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_set_index_1000",
            "value": 605020,
            "unit": "ns/iter"
          },
          {
            "name": "vm_comparison_ops_5000",
            "value": 2809700,
            "unit": "ns/iter"
          },
          {
            "name": "vm_equality_check_5000",
            "value": 1338500,
            "unit": "ns/iter"
          },
          {
            "name": "vm_string_concat_100",
            "value": 43172,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/100",
            "value": 42537,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/1000",
            "value": 351340,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/5000",
            "value": 1727200,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/10000",
            "value": 3446900,
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}
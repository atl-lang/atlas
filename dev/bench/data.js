window.BENCHMARK_DATA = {
  "lastUpdate": 1773585650508,
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
      },
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
          "id": "f72ded4cb90cb81d47a7f23f8f469b4c1c600674",
          "message": "ci: bump MSRV to 1.88 and fix vm_fuzz SecurityContext call [skip ci]",
          "timestamp": "2026-02-24T06:25:23Z",
          "url": "https://github.com/atl-lang/atlas/commit/f72ded4cb90cb81d47a7f23f8f469b4c1c600674"
        },
        "date": 1771957719760,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "interp_fibonacci_20",
            "value": 38759000,
            "unit": "ns/iter"
          },
          {
            "name": "interp_arrays/len_10k",
            "value": 6033400,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/fibonacci_20",
            "value": 8319800.000000001,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/nested_loops",
            "value": 3331200,
            "unit": "ns/iter"
          },
          {
            "name": "string_replace_large",
            "value": 62332,
            "unit": "ns/iter"
          },
          {
            "name": "array_map_10k_elements",
            "value": 62578,
            "unit": "ns/iter"
          },
          {
            "name": "array_sort_1000_numbers",
            "value": 59567,
            "unit": "ns/iter"
          },
          {
            "name": "array_indexOf_search",
            "value": 58810,
            "unit": "ns/iter"
          },
          {
            "name": "math_arithmetic_10k_ops",
            "value": 7496000,
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_10kb",
            "value": 68020,
            "unit": "ns/iter"
          },
          {
            "name": "json_nested_access_deep",
            "value": 68502,
            "unit": "ns/iter"
          },
          {
            "name": "file_read_1mb",
            "value": 161280,
            "unit": "ns/iter"
          },
          {
            "name": "file_write_1mb",
            "value": 6258600,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/10",
            "value": 125390,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/50",
            "value": 412390,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/200",
            "value": 1397000,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/50",
            "value": 155670,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/200",
            "value": 433470,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/500",
            "value": 1001400.0000000001,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/lex",
            "value": 1923.8999999999999,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/parse",
            "value": 3753.6,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/compile",
            "value": 5005.7,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/len_5k",
            "value": 2679600,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/trim_3k",
            "value": 604990,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/split_1k",
            "value": 1205900,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/push_1k",
            "value": 21036,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/pop_500",
            "value": 31915,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/len_5k",
            "value": 2650100,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/index_5k",
            "value": 2998100,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/abs_5k",
            "value": 2725300,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/sqrt_2k",
            "value": 1044100,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/min_max_3k",
            "value": 1993500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/type/type_of_3k",
            "value": 23943,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_add_1000",
            "value": 352220,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_sub_1000",
            "value": 350840,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_mul_1000",
            "value": 320700,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_div_1000",
            "value": 323310,
            "unit": "ns/iter"
          },
          {
            "name": "vm_function_multi_arg",
            "value": 411180,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_count_10000",
            "value": 1923800,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_accumulate_5000",
            "value": 1880500,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_nested_100x100",
            "value": 3128700,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_create_20",
            "value": 10750,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_set_index_1000",
            "value": 622840,
            "unit": "ns/iter"
          },
          {
            "name": "vm_comparison_ops_5000",
            "value": 2816900,
            "unit": "ns/iter"
          },
          {
            "name": "vm_equality_check_5000",
            "value": 1341000,
            "unit": "ns/iter"
          },
          {
            "name": "vm_string_concat_100",
            "value": 43380,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/100",
            "value": 42976,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/1000",
            "value": 354440,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/5000",
            "value": 1737900,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/10000",
            "value": 3469100,
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "proxikal@gmail.com",
            "name": "Joshua Cleland",
            "username": "proxikal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dcf7b257ce465dfaf4aa215a62cd60418516ed10",
          "message": "fix(compiler): return diagnostic instead of panic when TypeTag is unset (#155)\n\ncompiler/expr.rs:264 used .expect() on MemberExpr.type_tag, which panics\nwhen the typechecker leaves it as None (non-Array/JsonValue method calls pass\ntypecheck without error but without setting the tag). Converts the expect to\nok_or_else returning a Diagnostic, making the compiler surface an error\ngracefully instead of crashing.\n\nCaught by vm_fuzz nightly run (2026-02-26). Regression test added.\n\nCo-authored-by: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-02-26T23:45:53-05:00",
          "tree_id": "e31e932b37278688980f38078226c7c9f2842cfe",
          "url": "https://github.com/atl-lang/atlas/commit/dcf7b257ce465dfaf4aa215a62cd60418516ed10"
        },
        "date": 1772170580861,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "interp_fibonacci_20",
            "value": 38623000,
            "unit": "ns/iter"
          },
          {
            "name": "interp_arrays/len_10k",
            "value": 6043400,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/fibonacci_20",
            "value": 8196999.999999999,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/nested_loops",
            "value": 3300500,
            "unit": "ns/iter"
          },
          {
            "name": "string_replace_large",
            "value": 61905,
            "unit": "ns/iter"
          },
          {
            "name": "array_map_10k_elements",
            "value": 62886,
            "unit": "ns/iter"
          },
          {
            "name": "array_sort_1000_numbers",
            "value": 60762,
            "unit": "ns/iter"
          },
          {
            "name": "array_indexOf_search",
            "value": 59896,
            "unit": "ns/iter"
          },
          {
            "name": "math_arithmetic_10k_ops",
            "value": 7521400,
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_10kb",
            "value": 67827,
            "unit": "ns/iter"
          },
          {
            "name": "json_nested_access_deep",
            "value": 68266,
            "unit": "ns/iter"
          },
          {
            "name": "file_read_1mb",
            "value": 160550,
            "unit": "ns/iter"
          },
          {
            "name": "file_write_1mb",
            "value": 7511000,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/10",
            "value": 123600,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/50",
            "value": 404570,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/200",
            "value": 1372100,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/50",
            "value": 153660,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/200",
            "value": 424730,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/500",
            "value": 993160,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/lex",
            "value": 1859.2,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/parse",
            "value": 3742.4,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/compile",
            "value": 5100.3,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/len_5k",
            "value": 2593700,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/trim_3k",
            "value": 595890,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/split_1k",
            "value": 1218000,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/push_1k",
            "value": 21241,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/pop_500",
            "value": 34945,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/len_5k",
            "value": 2664700,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/index_5k",
            "value": 3046500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/abs_5k",
            "value": 2680000,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/sqrt_2k",
            "value": 1028400,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/min_max_3k",
            "value": 1993700,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/type/type_of_3k",
            "value": 23142,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_add_1000",
            "value": 358380,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_sub_1000",
            "value": 353540,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_mul_1000",
            "value": 324680,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_div_1000",
            "value": 329550,
            "unit": "ns/iter"
          },
          {
            "name": "vm_function_multi_arg",
            "value": 399400,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_count_10000",
            "value": 1917300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_accumulate_5000",
            "value": 1890000,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_nested_100x100",
            "value": 3142700,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_create_20",
            "value": 11019,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_set_index_1000",
            "value": 625460,
            "unit": "ns/iter"
          },
          {
            "name": "vm_comparison_ops_5000",
            "value": 2860900,
            "unit": "ns/iter"
          },
          {
            "name": "vm_equality_check_5000",
            "value": 1344800,
            "unit": "ns/iter"
          },
          {
            "name": "vm_string_concat_100",
            "value": 44395,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/100",
            "value": 44119,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/1000",
            "value": 358550,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/5000",
            "value": 1765800,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/10000",
            "value": 3518400,
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "proxikal@gmail.com",
            "name": "Joshua Cleland",
            "username": "proxikal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "c365fd27b6134c781b7a0c42df699bc6a8e07c75",
          "message": "feat(stdlib): systems-level modules — crypto, encoding, net, sync, websocket (#167)\n\n* feat(stdlib): add systems-level modules — crypto, encoding, net, sync, websocket\n\nCloses critical P1/P2 gaps identified in systems-language audit:\n\nCrypto (8 functions): sha256, sha512, blake3Hash, hmacSha256,\n  hmacSha256Verify, aesGcmEncrypt, aesGcmDecrypt, aesGcmGenerateKey\n\nEncoding (8 functions): base64Encode/Decode, base64UrlEncode/Decode,\n  hexEncode/Decode, urlEncode/Decode\n\nNetworking (22 functions):\n  TCP client: tcpConnect, tcpWrite, tcpRead, tcpReadBytes, tcpClose,\n    tcpSetTimeout, tcpSetNodelay, tcpLocalAddr, tcpRemoteAddr\n  TCP server: tcpListen, tcpAccept, tcpListenerAddr, tcpListenerClose\n  UDP: udpBind, udpSend, udpReceive, udpSetTimeout, udpClose, udpLocalAddr\n  TLS: tlsConnect, tlsWrite, tlsRead, tlsClose\n\nSync primitives (16 functions):\n  RwLock: rwLockNew, rwLockRead, rwLockWrite, rwLockTryRead, rwLockTryWrite\n  Semaphore: semaphoreNew, semaphoreAcquire, semaphoreTryAcquire,\n    semaphoreRelease, semaphoreAvailable\n  Atomic: atomicNew, atomicLoad, atomicStore, atomicAdd, atomicSub,\n    atomicCompareExchange\n\nWebSocket (6 functions): wsConnect, wsSend, wsSendBinary, wsReceive,\n  wsPing, wsClose\n\nAll modules:\n- Respect SecurityContext permission model for network ops\n- Use handle-based resource management (array-tagged IDs)\n- Follow existing stdlib patterns (arity checks, proper errors)\n- 8248 tests pass, 0 failures, 0 clippy warnings\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs: add systems-level audit handoff for next agent session\n\nComprehensive handoff covering completed work (60 new stdlib functions),\nremaining priorities (struct/enum types, ? operator, JIT integration),\nand standards for continuation.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-02T03:00:31-05:00",
          "tree_id": "84d416ee517c058788959aa2ec66d9ec3aa48b5c",
          "url": "https://github.com/atl-lang/atlas/commit/c365fd27b6134c781b7a0c42df699bc6a8e07c75"
        },
        "date": 1772441545184,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "interp_fibonacci_20",
            "value": 38705000,
            "unit": "ns/iter"
          },
          {
            "name": "interp_arrays/len_10k",
            "value": 6035100,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/fibonacci_20",
            "value": 8365399.999999999,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/nested_loops",
            "value": 3424100,
            "unit": "ns/iter"
          },
          {
            "name": "string_replace_large",
            "value": 62051,
            "unit": "ns/iter"
          },
          {
            "name": "array_map_10k_elements",
            "value": 62873,
            "unit": "ns/iter"
          },
          {
            "name": "array_sort_1000_numbers",
            "value": 60300,
            "unit": "ns/iter"
          },
          {
            "name": "array_indexOf_search",
            "value": 59325,
            "unit": "ns/iter"
          },
          {
            "name": "math_arithmetic_10k_ops",
            "value": 7569900,
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_10kb",
            "value": 67853,
            "unit": "ns/iter"
          },
          {
            "name": "json_nested_access_deep",
            "value": 68690,
            "unit": "ns/iter"
          },
          {
            "name": "file_read_1mb",
            "value": 157110,
            "unit": "ns/iter"
          },
          {
            "name": "file_write_1mb",
            "value": 6480100,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/10",
            "value": 122280,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/50",
            "value": 404690,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/200",
            "value": 1384900,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/50",
            "value": 153320,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/200",
            "value": 425660,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/500",
            "value": 986030,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/lex",
            "value": 1849.6999999999998,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/parse",
            "value": 3794.7000000000003,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/compile",
            "value": 5707.3,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/len_5k",
            "value": 2624500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/trim_3k",
            "value": 598010,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/split_1k",
            "value": 1147000,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/push_1k",
            "value": 21019,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/pop_500",
            "value": 34293,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/len_5k",
            "value": 2628000,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/index_5k",
            "value": 2977500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/abs_5k",
            "value": 2691400,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/sqrt_2k",
            "value": 1038500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/min_max_3k",
            "value": 2188500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/type/type_of_3k",
            "value": 23053,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_add_1000",
            "value": 360200,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_sub_1000",
            "value": 354700,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_mul_1000",
            "value": 326370,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_div_1000",
            "value": 324730,
            "unit": "ns/iter"
          },
          {
            "name": "vm_function_multi_arg",
            "value": 400060,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_count_10000",
            "value": 1912900,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_accumulate_5000",
            "value": 1870300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_nested_100x100",
            "value": 3207900,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_create_20",
            "value": 10808,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_set_index_1000",
            "value": 605990,
            "unit": "ns/iter"
          },
          {
            "name": "vm_comparison_ops_5000",
            "value": 2785400,
            "unit": "ns/iter"
          },
          {
            "name": "vm_equality_check_5000",
            "value": 1343300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_string_concat_100",
            "value": 43340,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/100",
            "value": 43398,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/1000",
            "value": 352870,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/5000",
            "value": 1733300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/10000",
            "value": 3450800,
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "proxikal@gmail.com",
            "name": "Joshua Cleland",
            "username": "proxikal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "0aad1edd790967aed322faa6ca2c933e4da54a5f",
          "message": "feat(parser): complete let mut syntax and var deprecation warning (#168)\n\n* scaffold(block-6): error handling — 5 phases planned\n\nExisting ? operator infrastructure verified:\n- Result ? works in interpreter, compiler, typechecker\n- VM has all required opcodes (IsResultOk, ExtractResultValue, etc.)\n- 10+ interpreter tests exist\n\nGaps identified and phased:\n1. Option ? support (typechecker/interpreter/compiler)\n2. Error type compatibility validation\n3. VM parity tests (none exist)\n4. Stdlib Result audit (≥20 fns)\n5. Integration + edge case tests\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* feat(block-6): add Option ? support — typechecker, interpreter, compiler\n\nPhase 01: extend ? operator from Result-only to also support Option<T>.\n\nChanges:\n- ast.rs: add TryTargetKind enum (Result|Option), TryExpr.target_kind annotation\n- typechecker: check_try() accepts Option<T>, validates function returns Option\n- interpreter: eval_try() handles Value::Option(Some/None)\n- compiler: compile_try() emits IsOptionSome/ExtractOptionValue for Option targets\n- 7 new tests for Option ? (unwrap Some, propagate None, nested, expressions)\n\n8,255 tests pass, 0 failures, 0 clippy warnings.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* feat(block-6): VM parity tests for ? operator — Result + Option\n\nPhase 03: 16 VM-specific tests for ? operator through full pipeline\n(Lexer → Parser → Binder → TypeChecker → Compiler → VM).\n\n- Added compile_checked() helper (includes binder+typechecker for AST annotations)\n- 7 Result ? VM tests (unwrap Ok, propagate Err, multiple, early return, nested)\n- 5 Option ? VM tests (unwrap Some, propagate None, multiple, early None, nested)\n- 4 parity tests (VM vs interpreter produce identical output)\n\n8,271 tests pass, 0 failures, 0 clippy warnings.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* feat(block-6): integration tests + fix ? in binary expressions\n\nPhase 05: integration tests for ? operator cross-feature interactions.\n\nBug fix: interpreter did not check ControlFlow after evaluating left\nside of binary expressions. When a()? + b()? had a() return Err,\nthe ? set ControlFlow::Return but + still tried to evaluate, causing\n\"Invalid operands for +\" error instead of propagating the Err.\n\nFix: check ControlFlow::None after each sub-expression in eval_binary.\n\nTests added:\n- 8 interpreter integration tests (multiple ? in expr, if conditions,\n  chained transforms, Option multiple ?)\n- 6 VM integration tests (multiple ?, chained, if conditions)\n- 5 VM parity integration tests\n\n8,285 tests pass, 0 failures, 0 clippy warnings.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs: update STATUS.md + add Block 6 handoff for next agent session\n\nBlock 6 progress: 4/5 phases complete.\nRemaining: Phase 04 (stdlib Result audit — ≥20 functions).\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* feat(block-6): stdlib Result/Option audit — 21+ functions converted\n\nPhase 04: Convert panicking/sentinel stdlib functions to Result<T,E>\nor Option<T> for safe error handling with the ? operator.\n\nOption conversions: indexOf, lastIndexOf, arrayIndexOf, arrayLastIndexOf,\ncharAt, find, findIndex, getEnv (sentinel -1/null → Option<T>)\n\nResult conversions: toNumber, parseInt, parseFloat, parseJSON, sqrt,\nlog, asin, acos, clamp (panics/NaN → Result<T, String>)\n\nInternal safety: toJSON serde unwrap, 3x fs.rs SystemTime unwraps\n\nAlso: top-level ? operator support (typechecker + both engines),\nsymbol.rs type declarations updated, all 8,285 tests updated and passing.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(tests): remove unused test helpers and imports\n\nAll 11 open PRs were failing Clippy due to unused code:\n- Removed unused assert_parity() helper (mutations.rs)\n- Removed unused run_interpreter() helper (interpreter.rs)\n- Removed 4 test functions missing #[test] attributes\n- Removed 3 unused pretty_assertions imports\n\nFixes CI failures for PRs #157-166.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs: add pre-systems hardening roadmap and STATUS.md integration\n\nCreates comprehensive hardening tracker for post-v0.3 work:\n- Maps 7 critical gaps from audit findings\n- Defines 5 hardening phases (H1-H5) before systems-level conversion\n- Sets acceptance criteria for hardening completion\n- Links to advanced-codex-audit.md and codex-findings/\n\nUpdates STATUS.md:\n- Adds prominent hardening notice in Current State section\n- Links PRE-SYSTEMS-HARDENING.md in Quick Links\n- Clarifies: language functionality > code hygiene\n- Instructs AI agents: complete v0.3 → hardening → systems-level\n\nPriority: Get scripting-level language stable before adding borrow\nchecker, lifetimes, or manual memory management.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* fix(docs): correct v0.3 scope — hardening is part of v0.3, not post-v0.3\n\nTerminology fixes:\n- v0.3 = language completeness + hardening (open-ended)\n- v0.4+ = systems-level (borrow checker, AOT)\n- Hardening phases H1-H5 are PART OF v0.3, not after it\n\nClarifications:\n- Blocks 1-9 are first milestone, not entire v0.3\n- v0.3 complete when acceptance criteria met (tens/hundreds of blocks)\n- Then tag v0.3.0 and begin v0.4 systems-level work\n\nUpdated PRE-SYSTEMS-HARDENING.md title to 'v0.3 Hardening Roadmap\n(Post-v0.2 Stabilization)' and adjusted all references.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* revert: remove over-engineered hardening doc\n\nRemoved PRE-SYSTEMS-HARDENING.md - too much forward-looking complexity.\nSimplified STATUS.md - removed confusing v0.4 references.\n\nFocus: Fix what's broken NOW. Audit findings already document gaps.\nDon't project into future versions.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(STATUS): add deferred work notes (inline tests + systems-level)\n\n- Inline tests: ~574 to audit after hardening complete\n- Systems-level: blocked on core language compiler readiness\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(STATUS): link systems-level conversion to ROADMAP\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(STATUS): clarify v0.3 scope — all systems-level work in v0.3\n\n- Block 5 (Type Inference) = last systems-level block done\n- Systems-level conversion PAUSED until hardening complete\n- v0.3 is open-ended: features + hardening + systems-level\n- No version increment until compiler is professionally complete\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(STATUS): clarify v0.3 exit criteria - fix current features, not build forever\n\nv0.3 = make what exists work correctly, not feature-complete compiler.\nBattle-test current foundation before future feature additions.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(STATUS): track current hardening work from audit findings\n\nPointers to:\n- advanced-codex-audit.md (battle-test findings)\n- systems-audit-handoff.md (stdlib + priorities)\n\nAI agents: alert user when hardening 100% complete (more audits pending).\nFeature blocks (7-9) remain visible for post-hardening continuation.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs: add Claude audit findings to tracking (2026-03-02)\n\nMoved CLAUDE-ATLAS-AUDIT-03-02-26.md to docs/codex-findings/claude-audit-2026-03-02.md\nAdded to STATUS.md hardening work tracking.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* feat(parser): complete let mut syntax and var deprecation warning\n\nImplements Rust-style `let mut` for mutable variable declarations and\nadds AT2014 deprecation warning when the legacy `var` keyword is used.\n\nChanges:\n- Add `mut` keyword token and parser support for `let mut x = ...`\n- Track deprecated `var` usage via `uses_deprecated_var` AST field\n- Emit AT2014 warning in typechecker when `var` is detected\n- Fix runtime/CLI to distinguish warnings from errors (only fail on errors)\n- Update suggestions to recommend `let mut` instead of `var`\n- Add comprehensive runtime tests for let mut behavior\n- Update all existing tests to use `let mut` syntax\n- Update syntax.md specification\n\nThe deprecation warning provides a smooth migration path while maintaining\nbackwards compatibility. `var` still works but emits a clear warning.\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* docs(STATUS): record let mut hardening completion\n\n- Updated test count: 8,302 (up from 8,285)\n- AT2014 deprecation warning for `var` now functional\n- let mut syntax fully operational\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n* style: cargo fmt diagnostics.rs\n\nCo-Authored-By: Claude Opus 4.6 <noreply@anthropic.com>\n\n---------\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-02T20:22:07Z",
          "tree_id": "39123565fea6911b4b2f1d215b1554e7b7b8c8f9",
          "url": "https://github.com/atl-lang/atlas/commit/0aad1edd790967aed322faa6ca2c933e4da54a5f"
        },
        "date": 1772485984595,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "interp_fibonacci_20",
            "value": 38508000,
            "unit": "ns/iter"
          },
          {
            "name": "interp_arrays/len_10k",
            "value": 6031500,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/fibonacci_20",
            "value": 8202400.000000001,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/nested_loops",
            "value": 3312200,
            "unit": "ns/iter"
          },
          {
            "name": "string_replace_large",
            "value": 66251,
            "unit": "ns/iter"
          },
          {
            "name": "array_map_10k_elements",
            "value": 66053,
            "unit": "ns/iter"
          },
          {
            "name": "array_sort_1000_numbers",
            "value": 64155,
            "unit": "ns/iter"
          },
          {
            "name": "array_indexOf_search",
            "value": 63512,
            "unit": "ns/iter"
          },
          {
            "name": "math_arithmetic_10k_ops",
            "value": 7593300,
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_10kb",
            "value": 71565,
            "unit": "ns/iter"
          },
          {
            "name": "json_nested_access_deep",
            "value": 68988,
            "unit": "ns/iter"
          },
          {
            "name": "file_read_1mb",
            "value": 156810,
            "unit": "ns/iter"
          },
          {
            "name": "file_write_1mb",
            "value": 6473400,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/10",
            "value": 124740,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/50",
            "value": 409250,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/200",
            "value": 1394400,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/50",
            "value": 156260,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/200",
            "value": 432720,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/500",
            "value": 1005500.0000000001,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/lex",
            "value": 1847.6999999999998,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/parse",
            "value": 3716,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/compile",
            "value": 5447.099999999999,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/len_5k",
            "value": 2583200,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/trim_3k",
            "value": 593690,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/split_1k",
            "value": 1230900,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/push_1k",
            "value": 21526,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/pop_500",
            "value": 32429.000000000004,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/len_5k",
            "value": 2614000,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/index_5k",
            "value": 2976800,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/abs_5k",
            "value": 2710800,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/sqrt_2k",
            "value": 20082,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/min_max_3k",
            "value": 1994400,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/type/type_of_3k",
            "value": 24502,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_add_1000",
            "value": 353650,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_sub_1000",
            "value": 349090,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_mul_1000",
            "value": 319460,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_div_1000",
            "value": 325630,
            "unit": "ns/iter"
          },
          {
            "name": "vm_function_multi_arg",
            "value": 399170,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_count_10000",
            "value": 1918300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_accumulate_5000",
            "value": 1890400,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_nested_100x100",
            "value": 3133600,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_create_20",
            "value": 11253,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_set_index_1000",
            "value": 614860,
            "unit": "ns/iter"
          },
          {
            "name": "vm_comparison_ops_5000",
            "value": 2788300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_equality_check_5000",
            "value": 1357800,
            "unit": "ns/iter"
          },
          {
            "name": "vm_string_concat_100",
            "value": 43769,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/100",
            "value": 43370,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/1000",
            "value": 353260,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/5000",
            "value": 1727500,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/10000",
            "value": 3451900,
            "unit": "ns/iter"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "proxikal@gmail.com",
            "name": "Joshua Cleland",
            "username": "proxikal"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "2bd238e8915348ec7587c9e0402a5f51abcc06cd",
          "message": "fix(http): enable member access on HttpResponse values (#171)\n\nHttpResponse values now support method-call syntax for accessing properties:\n- response.status()  → httpStatus(response)\n- response.body()    → httpBody(response)\n- response.headers() → httpHeaders(response)\n- response.url()     → httpUrl(response)\n- response.isSuccess() → httpIsSuccess(response)\n\nChanges:\n- Added TypeTag::HttpResponse to method dispatch system\n- Added resolve_http_response_method() with mappings\n- Added dynamic type tag detection for HttpResponse in interpreter\n\nCloses: H-013\n\nCo-authored-by: Claude Opus 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-03T00:43:21Z",
          "tree_id": "bcb5da627f7dfa5622b7ee9bd2b2e8692d49f680",
          "url": "https://github.com/atl-lang/atlas/commit/2bd238e8915348ec7587c9e0402a5f51abcc06cd"
        },
        "date": 1772501676348,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "interp_fibonacci_20",
            "value": 38609000,
            "unit": "ns/iter"
          },
          {
            "name": "interp_arrays/len_10k",
            "value": 5997700,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/fibonacci_20",
            "value": 8285800,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/nested_loops",
            "value": 3467800,
            "unit": "ns/iter"
          },
          {
            "name": "string_replace_large",
            "value": 66433,
            "unit": "ns/iter"
          },
          {
            "name": "array_map_10k_elements",
            "value": 65771,
            "unit": "ns/iter"
          },
          {
            "name": "array_sort_1000_numbers",
            "value": 63615,
            "unit": "ns/iter"
          },
          {
            "name": "array_indexOf_search",
            "value": 63011,
            "unit": "ns/iter"
          },
          {
            "name": "math_arithmetic_10k_ops",
            "value": 7685700,
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_10kb",
            "value": 70782,
            "unit": "ns/iter"
          },
          {
            "name": "json_nested_access_deep",
            "value": 68338,
            "unit": "ns/iter"
          },
          {
            "name": "file_read_1mb",
            "value": 161750,
            "unit": "ns/iter"
          },
          {
            "name": "file_write_1mb",
            "value": 6479700,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/10",
            "value": 126040,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/50",
            "value": 406590,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/200",
            "value": 1382600,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/50",
            "value": 156640,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/200",
            "value": 429350,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/500",
            "value": 990250,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/lex",
            "value": 1901.7,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/parse",
            "value": 3646.9,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/compile",
            "value": 5481,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/len_5k",
            "value": 2598400,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/trim_3k",
            "value": 590810,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/split_1k",
            "value": 1211100,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/push_1k",
            "value": 21499,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/pop_500",
            "value": 32491.999999999996,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/len_5k",
            "value": 2613700,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/index_5k",
            "value": 2971600,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/abs_5k",
            "value": 2729300,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/sqrt_2k",
            "value": 20013,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/min_max_3k",
            "value": 1999400,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/type/type_of_3k",
            "value": 23438,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_add_1000",
            "value": 356380,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_sub_1000",
            "value": 355290,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_mul_1000",
            "value": 324360,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_div_1000",
            "value": 324680,
            "unit": "ns/iter"
          },
          {
            "name": "vm_function_multi_arg",
            "value": 396820,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_count_10000",
            "value": 1919900,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_accumulate_5000",
            "value": 1887100,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_nested_100x100",
            "value": 3161500,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_create_20",
            "value": 10970,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_set_index_1000",
            "value": 603270,
            "unit": "ns/iter"
          },
          {
            "name": "vm_comparison_ops_5000",
            "value": 2808800,
            "unit": "ns/iter"
          },
          {
            "name": "vm_equality_check_5000",
            "value": 1340200,
            "unit": "ns/iter"
          },
          {
            "name": "vm_string_concat_100",
            "value": 43720,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/100",
            "value": 44088,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/1000",
            "value": 361140,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/5000",
            "value": 1774300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/10000",
            "value": 3537600,
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
          "id": "550011ffe22935848a315ece411665b401d7bd27",
          "message": "chore(tracking): update issue status for H-004",
          "timestamp": "2026-03-02T21:53:45-05:00",
          "tree_id": "e0aa38f4cac1df4ff7e41f71d0ef16390b15dbc5",
          "url": "https://github.com/atl-lang/atlas/commit/550011ffe22935848a315ece411665b401d7bd27"
        },
        "date": 1772509627316,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "interp_fibonacci_20",
            "value": 38902000,
            "unit": "ns/iter"
          },
          {
            "name": "interp_arrays/len_10k",
            "value": 6153800,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/fibonacci_20",
            "value": 8276899.999999999,
            "unit": "ns/iter"
          },
          {
            "name": "parity/vm/nested_loops",
            "value": 3601800,
            "unit": "ns/iter"
          },
          {
            "name": "string_replace_large",
            "value": 65394.00000000001,
            "unit": "ns/iter"
          },
          {
            "name": "array_map_10k_elements",
            "value": 65111.99999999999,
            "unit": "ns/iter"
          },
          {
            "name": "array_sort_1000_numbers",
            "value": 62475,
            "unit": "ns/iter"
          },
          {
            "name": "array_indexOf_search",
            "value": 62239,
            "unit": "ns/iter"
          },
          {
            "name": "math_arithmetic_10k_ops",
            "value": 7802600,
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_10kb",
            "value": 70334,
            "unit": "ns/iter"
          },
          {
            "name": "json_nested_access_deep",
            "value": 66922,
            "unit": "ns/iter"
          },
          {
            "name": "file_read_1mb",
            "value": 160380,
            "unit": "ns/iter"
          },
          {
            "name": "file_write_1mb",
            "value": 7778400,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/10",
            "value": 125760,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/50",
            "value": 413230,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/200",
            "value": 1396200,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/50",
            "value": 157500,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/200",
            "value": 434670,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/500",
            "value": 995780,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/lex",
            "value": 1898.2,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/parse",
            "value": 3785.7,
            "unit": "ns/iter"
          },
          {
            "name": "pipeline/stages/compile",
            "value": 5604.4,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/len_5k",
            "value": 2893500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/trim_3k",
            "value": 685500,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/split_1k",
            "value": 1274200,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/push_1k",
            "value": 21294,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/pop_500",
            "value": 33424,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/len_5k",
            "value": 2943700,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/index_5k",
            "value": 3326800,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/abs_5k",
            "value": 3032700,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/sqrt_2k",
            "value": 21224,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/min_max_3k",
            "value": 2178100,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/type/type_of_3k",
            "value": 24914,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_add_1000",
            "value": 382360,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_sub_1000",
            "value": 380300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_mul_1000",
            "value": 349730,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_div_1000",
            "value": 350120,
            "unit": "ns/iter"
          },
          {
            "name": "vm_function_multi_arg",
            "value": 449780,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_count_10000",
            "value": 2100700,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_accumulate_5000",
            "value": 1999400,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_nested_100x100",
            "value": 3422000,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_create_20",
            "value": 11025,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_set_index_1000",
            "value": 648770,
            "unit": "ns/iter"
          },
          {
            "name": "vm_comparison_ops_5000",
            "value": 3070600,
            "unit": "ns/iter"
          },
          {
            "name": "vm_equality_check_5000",
            "value": 1442600,
            "unit": "ns/iter"
          },
          {
            "name": "vm_string_concat_100",
            "value": 46639,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/100",
            "value": 46091,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/1000",
            "value": 384000,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/5000",
            "value": 1872400,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/10000",
            "value": 3738800,
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
          "id": "482a62045a482ee58e36143dfbd2a25f6933c0a2",
          "message": "fix(web): template map reassignment and truthy eval_condition\n\n- item_ctx.set() is non-mutating — assign result back (CoW)\n- eval_condition: treat false/0/\"\"/null as falsy, not just None\n\nCo-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>",
          "timestamp": "2026-03-15T10:10:05-04:00",
          "tree_id": "0eeb715410a8b4543c2b5fa3443994bb0a9e604f",
          "url": "https://github.com/atl-lang/atlas/commit/482a62045a482ee58e36143dfbd2a25f6933c0a2"
        },
        "date": 1773585649978,
        "tool": "customSmallerIsBetter",
        "benches": [
          {
            "name": "string_replace_large",
            "value": 120220,
            "unit": "ns/iter"
          },
          {
            "name": "array_map_10k_elements",
            "value": 18896,
            "unit": "ns/iter"
          },
          {
            "name": "array_sort_1000_numbers",
            "value": 24511,
            "unit": "ns/iter"
          },
          {
            "name": "array_indexOf_search",
            "value": 21723,
            "unit": "ns/iter"
          },
          {
            "name": "math_arithmetic_10k_ops",
            "value": 21509,
            "unit": "ns/iter"
          },
          {
            "name": "json_parse_10kb",
            "value": 214200,
            "unit": "ns/iter"
          },
          {
            "name": "json_nested_access_deep",
            "value": 229000,
            "unit": "ns/iter"
          },
          {
            "name": "file_read_1mb",
            "value": 211330,
            "unit": "ns/iter"
          },
          {
            "name": "file_write_1mb",
            "value": 4457100,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/10",
            "value": 191190,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/50",
            "value": 747280,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_functions/200",
            "value": 3910300,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/50",
            "value": 225010,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/200",
            "value": 714470,
            "unit": "ns/iter"
          },
          {
            "name": "typecheck_scopes/500",
            "value": 1508300,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/len_5k",
            "value": 3376200,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/trim_3k",
            "value": 831470,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/string/split_1k",
            "value": 2285400,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/push_1k",
            "value": 39570,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/pop_500",
            "value": 58584,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/len_5k",
            "value": 47837,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/array/index_5k",
            "value": 45874,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/abs_5k",
            "value": 41096,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/sqrt_2k",
            "value": 41138,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/math/min_max_3k",
            "value": 45752,
            "unit": "ns/iter"
          },
          {
            "name": "stdlib/type/type_of_3k",
            "value": 3078400,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_add_1000",
            "value": 474680,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_sub_1000",
            "value": 476900,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_mul_1000",
            "value": 462830,
            "unit": "ns/iter"
          },
          {
            "name": "vm_arithmetic_div_1000",
            "value": 451210,
            "unit": "ns/iter"
          },
          {
            "name": "vm_function_multi_arg",
            "value": 682840,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_count_10000",
            "value": 2691900,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_accumulate_5000",
            "value": 2602400,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_nested_100x100",
            "value": 4303300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_create_20",
            "value": 13108,
            "unit": "ns/iter"
          },
          {
            "name": "vm_array_set_index_1000",
            "value": 944940,
            "unit": "ns/iter"
          },
          {
            "name": "vm_comparison_ops_5000",
            "value": 3830200,
            "unit": "ns/iter"
          },
          {
            "name": "vm_equality_check_5000",
            "value": 2026300,
            "unit": "ns/iter"
          },
          {
            "name": "vm_string_concat_100",
            "value": 67227,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/100",
            "value": 63072,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/1000",
            "value": 483800,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/5000",
            "value": 2344600,
            "unit": "ns/iter"
          },
          {
            "name": "vm_loop_scaling/10000",
            "value": 4677700,
            "unit": "ns/iter"
          }
        ]
      }
    ]
  }
}
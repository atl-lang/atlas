use super::super::*;
// ============================================================================

#[test]
fn test_text_word_count() {
    let code = r#"
        let text: string = "hello world this is a test";
        let words: string[] = split(text, " ");
        len(words)
    "#;
    assert_eval_number_with_io(code, 6.0);
}

#[test]
fn test_text_line_count() {
    let code = r#"
        let text: string = "line1\nline2\nline3";
        let lines: string[] = split(text, "\n");
        len(lines)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_text_average_word_length() {
    let code = r#"
        fn wordLength(word: string) -> number { return len(word); }
        fn sum(a: number, b: number) -> number { return a + b; }

        let text: string = "the quick brown fox";
        let words: string[] = split(text, " ");
        let lengths: number[] = map(words, wordLength);
        let total: number = reduce(lengths, sum, 0.0);
        let avg: number = total / len(words);
        floor(avg)
    "#;
    assert_eval_number_with_io(code, 4.0); // (3+5+5+3)/4 = 4
}

#[test]
fn test_text_uppercase_words() {
    let code = r#"
        fn toUpper(s: string) -> string { return toUpperCase(s); }

        let text: string = "hello world";
        let words: string[] = split(text, " ");
        let uppered: string[] = map(words, toUpper);
        join(uppered, " ")
    "#;
    assert_eval_string_with_io(code, "HELLO WORLD");
}

#[test]
fn test_text_titlecase() {
    let code = r#"
        fn titleCase(word: string) -> string {
            let first: string = charAt(word, 0.0);
            let rest: string = substring(word, 1.0, len(word));
            let firstUpper: string = toUpperCase(first);
            let restLower: string = toLowerCase(rest);
            return firstUpper + restLower;
        }

        let text: string = "hello WORLD";
        let words: string[] = split(text, " ");
        let titled: string[] = map(words, titleCase);
        join(titled, " ")
    "#;
    assert_eval_string_with_io(code, "Hello World");
}

#[test]
fn test_text_remove_punctuation() {
    let code = r#"
        fn removePunct(s: string) -> string {
            let s1: string = replace(s, ".", "");
            let s2: string = replace(s1, ",", "");
            let s3: string = replace(s2, "!", "");
            return s3;
        }

        let text: string = "Hello, World! Test.";
        removePunct(text)
    "#;
    assert_eval_string_with_io(code, "Hello World Test");
}

#[test]
fn test_text_find_longest_word() {
    let code = r#"
        fn longerWord(current: string, word: string) -> string {
            if (len(word) > len(current)) {
                return word;
            }
            return current;
        }

        let text: string = "the quick brown fox jumps";
        let words: string[] = split(text, " ");
        reduce(words, longerWord, "")
    "#;
    assert_eval_string_with_io(code, "quick"); // or "brown" or "jumps" (all 5 chars, first wins)
}

#[test]
fn test_text_filter_short_words() {
    let code = r#"
        fn isLong(word: string) -> bool {
            return len(word) >= 4.0;
        }

        let text: string = "the quick brown fox";
        let words: string[] = split(text, " ");
        let long: string[] = filter(words, isLong);
        len(long)
    "#;
    assert_eval_number_with_io(code, 2.0); // "quick"=5, "brown"=5 are >=4
}

#[test]
fn test_text_count_character() {
    let code = r#"
        let text: string = "hello world";
        let chars: string[] = split(text, "");
        fn isL(c: string) -> bool { return c == "l"; }
        let ls: string[] = filter(chars, isL);
        len(ls)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_text_reverse_words() {
    let code = r#"
        let text: string = "hello world";
        let words: string[] = split(text, " ");
        let reversed: string[] = reverse(words);
        join(reversed, " ")
    "#;
    assert_eval_string_with_io(code, "world hello");
}

#[test]
fn test_text_acronym() {
    let code = r#"
        fn firstChar(s: string) -> string {
            return charAt(s, 0.0);
        }

        let text: string = "Portable Network Graphics";
        let words: string[] = split(text, " ");
        let initials: string[] = map(words, firstChar);
        join(initials, "")
    "#;
    assert_eval_string_with_io(code, "PNG");
}

#[test]
fn test_text_trim_lines() {
    let code = r#"
        fn trimLine(line: string) -> string { return trim(line); }

        let text: string = "  line1  \n  line2  \n  line3  ";
        let lines: string[] = split(text, "\n");
        let trimmed: string[] = map(lines, trimLine);
        join(trimmed, "|")
    "#;
    assert_eval_string_with_io(code, "line1|line2|line3");
}

#[test]
fn test_text_starts_with_filter() {
    let code = r#"
        fn startsWithA(word: string) -> bool {
            return startsWith(word, "a");
        }

        let words: string[] = ["apple", "banana", "apricot", "cherry"];
        let aWords: string[] = filter(words, startsWithA);
        len(aWords)
    "#;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_text_ends_with_filter() {
    let code = r#"
        fn endsWithE(word: string) -> bool {
            return endsWith(word, "e");
        }

        let words: string[] = ["apple", "banana", "grape", "cherry"];
        let eWords: string[] = filter(words, endsWithE);
        len(eWords)
    "#;
    assert_eval_number_with_io(code, 2.0); // apple, grape
}

#[test]
fn test_text_pad_lines() {
    let code = r#"
        fn pad(line: string) -> string {
            return padEnd(line, 10.0, ".");
        }

        let lines: string[] = ["short", "medium", "long"];
        let padded: string[] = map(lines, pad);
        padded[0]
    "#;
    assert_eval_string_with_io(code, "short.....");
}

#[test]
fn test_text_replace_multiple() {
    let code = r#"
        let text: string = "foo bar foo baz";
        let step1: string = replace(text, "foo", "hello");
        let step2: string = replace(step1, "bar", "world");
        step2
    "#;
    assert_eval_string_with_io(code, "hello world foo baz"); // only first "foo" replaced
}

#[test]
fn test_text_split_multichar() {
    let code = r#"
        let text: string = "one::two::three";
        let parts: string[] = split(text, "::");
        len(parts)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_text_extract_numbers() {
    let code = r#"
        let text: string = "Price: 100 Quantity: 50";
        let words: string[] = split(text, " ");
        let num1: number = parseFloat(words[1]);
        let num2: number = parseFloat(words[3]);
        num1 + num2
    "#;
    assert_eval_number_with_io(code, 150.0);
}

#[test]
fn test_text_repeat_pattern() {
    let code = r#"
        let pattern: string = repeat("*", 5.0);
        pattern
    "#;
    assert_eval_string_with_io(code, "*****");
}

#[test]
fn test_text_contains_substring() {
    let code = r#"
        let text: string = "The quick brown fox";
        let hasQuick: bool = includes(text, "quick");
        let hasSlow: bool = includes(text, "slow");
        hasQuick && !hasSlow
    "#;
    assert_eval_bool_with_io(code, true);
}

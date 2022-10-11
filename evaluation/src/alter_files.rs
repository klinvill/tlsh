use std::io;
use std::path::Path;
use rand::prelude::*;

struct AlteredText {
    text: Vec<char>,

    rng: ThreadRng,

    // Dictionary of english words to use when inserting a new word
    dict: Vec<String>,
}

impl AlteredText {
    fn new(text: Vec<char>) -> Self {
        let rng = thread_rng();
        let dict = std::fs::read_to_string("/usr/share/dict/words")
            .expect("Expected to read dictionary from /usr/share/dict/words")
            .lines()
            .map(|line| line.to_string())
            .collect();

        AlteredText {
            text,
            rng,
            dict,
        }
    }

    fn from_file(file: &Path) -> io::Result<Self> {
        let text = std::fs::read_to_string(file)?.chars().collect();
        Ok(AlteredText::new(text))
    }

    fn permute(&mut self, times: usize) {
        // Randomly select one of the possible permutations
        let operations = [
            AlteredText::insert_word,
            AlteredText::delete_word,
            AlteredText::swap_words,
            AlteredText::substitute_words,
            AlteredText::replace_chars,
            AlteredText::delete_chars,
        ];

        for _ in 0..times {
            let op = operations.choose(&mut self.rng).unwrap();
            op(self);
        }
    }

    fn is_valid_word_char(c: char) -> bool {
        return c.is_alphanumeric() || match c {
            // We consider apostrophes and dashes to be part of a word. E.g. they're, or
            '\'' | '-' => true,
            _ => false
        }
    }

    // Given an index into text, this function finds the word that i is in. The returned indices
    // are inclusive on the left and exclusive on the right.
    fn expand_word(&self, i: usize) -> Option<(usize, usize)> {
        if !AlteredText::is_valid_word_char(self.text[i]) {
            return None;
        }

        let (mut start, mut end) = (i, i+1);
        while start > 0 && AlteredText::is_valid_word_char(self.text[start-1]) {
            start -= 1;
        }
        while end < self.text.len() && AlteredText::is_valid_word_char(self.text[end]) {
            end += 1;
        }
        Some((start, end))
    }

    fn pick_random_word(&mut self) -> (usize, usize) {
        let mut try_word = None;
        // Keep sampling until we find a valid word
        while try_word.is_none() {
            let i = self.rng.gen_range(0..self.text.len());
            try_word = self.expand_word(i);
        }

        try_word.unwrap()
    }

    fn pick_random_char(&mut self) -> char {
        // We try to avoid replacing characters that would change the number of "words" in the text
        // since it would dramatically influence other transformations as the number of
        // transformations increases.
        let mut c = ' ';
        // Keep sampling until we find a valid word character.
        while !AlteredText::is_valid_word_char(c) {
            let i = self.rng.gen_range(0..self.text.len());
            c = self.text[i];
        }

        c
    }

    fn insert_word(&mut self) -> () {
        // Try to only insert before or after words
        let (start, end) = self.pick_random_word();

        let mut word = self.dict.choose(&mut self.rng).unwrap().to_string();

        let do_before = self.rng.gen::<bool>();
        let i = if do_before {
            start
        } else {
            end
        };

        // Pad word with spaces so it looks natural
        if do_before {
            word.push(' ');
        } else {
            word.insert(0, ' ');
        }

        self._insert_word(&word, i);
    }

    fn _insert_word(&mut self, word: &str, i: usize) -> () {
        for (j, c) in word.chars().enumerate() {
            self.text.insert(i+j, c);
        }
    }

    fn delete_word(&mut self) -> () {
        let (start, end) = self.pick_random_word();
        self._delete_word(start, end);
    }

    fn _delete_word(&mut self, start: usize, end: usize) -> () {
        self.text.drain(start..end);
    }

    fn swap_words(&mut self) -> () {
        let (start_1, end_1) = self.pick_random_word();
        let (start_2, end_2) = self.pick_random_word();
        self._swap_words(start_1, end_1, start_2, end_2);
    }

    fn _swap_words(&mut self, start_1: usize, end_1: usize, start_2: usize, end_2: usize) -> () {
        // Just ignore it if we happen to pick the same word
        if start_1 == start_2 {
            return;
        }

        let len_1 = end_1 - start_1;
        let len_2 = end_2 - start_2;

        let (longer_start, longer_end, shorter_start, shorter_end) = if len_1 > len_2 {
            (start_1, end_1, start_2, end_2)
        } else {
            (start_2, end_2, start_1, end_1)
        };

        let shorter_len = shorter_end - shorter_start;
        let longer_len = longer_end - longer_start;

        // Can do an efficient swap operation for up through the shortest string
        for i in 0..shorter_len {
            self.text.swap(shorter_start + i, longer_start + i);
        }

        // But then have to do a less efficient drain and insert with what remains
        let removed: Vec<char> = self.text.drain(longer_start+shorter_len..longer_end).collect();
        // If the removed text comes before where we're going to insert text, we need to adjust the index to account for the changes from the removal
        let adjust_index = if longer_start < shorter_start {
            longer_len - shorter_len
        } else {
            0
        };
        for (j, c) in removed.iter().enumerate() {
            self.text.insert(shorter_end - adjust_index + j, *c);
        }
    }

    fn substitute_words(&mut self) {
        let (start_1, end_1) = self.pick_random_word();
        let (start_2, end_2) = self.pick_random_word();
        self._substitute_words(start_1, end_1, start_2, end_2);
    }

    // Replaces the specific occurrence of the second word with the first word. E.g., substituting
    // "foo" for "bar" in the sentence "foo and bar are two placeholders" would result in the
    // sentence "foo and foo are two placeholders".
    fn _substitute_words(&mut self, start_1: usize, end_1: usize, start_2: usize, end_2: usize) {
        // Just ignore it if we happen to pick the same word
        if start_1 == start_2 {
            return;
        }

        let len_1 = end_1 - start_1;
        let len_2 = end_2 - start_2;

        // Can do an efficient replacement operation for up through the second string
        for i in 0..std::cmp::min(len_2, len_1) {
            self.text[start_2 + i] = self.text[start_1 + i]
        }

        // But any excess characters need to be inserted or deleted using a slower operation
        if len_1 > len_2 {
            for j in 0..len_1 - len_2 {
                self.text.insert(start_2 + len_2 + j, self.text[start_1 + len_2 + j]);
            }
        } else {
            self.text.drain(start_2 + len_1 + 0..start_2 + len_2);
        }
    }

    fn replace_chars(&mut self) {
        let target = self.pick_random_char();
        let source = self.pick_random_char();
        let start = self.rng.gen_range(0..self.text.len());

        // We try to replace 10 occurrences of the target character
        self._replace_chars(target, source, start, 10);
    }

    // Replaces the target character with the source character up to `times` occurrences starting
    // at the index start and wrapping back around to the beginning if needed.
    fn _replace_chars(&mut self, target: char, source: char, start: usize, times: usize) {
        let mut to_replace = times;

        if to_replace <= 0 {
            return;
        }

        for i in start..self.text.len() {
            let c = self.text[i];
            if c == target {
                self.text[i] = source;
                to_replace -= 1;

                // Are we done yet?
                if to_replace <= 0 {
                    return;
                }
            }
        }

        // If we haven't found all occurrences from start to the end yet, try looking further from
        // the beginning to start.
        for j in 0..start {
            let c = self.text[j];
            if c == target {
                self.text[j] = source;
                to_replace -= 1;

                // Are we done yet?
                if to_replace <= 0 {
                    return;
                }
            }
        }
    }

    fn delete_chars(&mut self) {
        let target = self.pick_random_char();
        let start = self.rng.gen_range(0..self.text.len());

        // We try to delete 10 occurrences of the target character
        self._delete_chars(target, start, 10);
    }

    // Deletes the target character up to `times` occurrences starting at the index start and
    // wrapping back around to the beginning if needed.
    fn _delete_chars(&mut self, target: char, start: usize, times: usize) {
        let mut indices_to_delete = Vec::with_capacity(times);

        if times <= 0 {
            return;
        }

        for i in start..self.text.len() {
            let c = self.text[i];
            if c == target {
                indices_to_delete.push(i);

                // Are we done yet?
                if indices_to_delete.len() >= times {
                    break;
                }
            }
        }

        // If we haven't found all occurrences from start to the end yet, try looking further from
        // the beginning to start.
        if indices_to_delete.len() < times {
            for j in 0..start {
                let c = self.text[j];
                if c == target {
                    indices_to_delete.push(j);

                    // Are we done yet?
                    if indices_to_delete.len() >= times {
                        break;
                    }
                }
            }
        }

        // Now delete all the elements in reverse order so we don't accidentally shift the indices
        indices_to_delete.sort();
        for i in indices_to_delete.iter().rev() {
            self.text.remove(*i);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_word() {
        let orig_text = "This is a semi-short test string (that'll test some edge cases).";
        let mut text = AlteredText::new(orig_text.chars().collect());
        let words_to_insert = [
            (20, " foo"),
            (38, "bar "),
            (71, " antidisestablishmentarianism"),
        ];

        let mut reference_text = orig_text.to_string();
        for (i, word) in words_to_insert {
            reference_text.insert_str(i, word);

            text._insert_word(word, i);
            assert_eq!(text.text, reference_text.chars().collect::<Vec<char>>())
        }
    }

    #[test]
    fn test_delete_word() {
        let orig_text = "This is a semi-short test string (that'll test some edge cases).";
        let mut text = AlteredText::new(orig_text.chars().collect());
        let words_to_remove = ["semi-short", "cases", "that'll"];

        let mut reference_text = orig_text.to_string();
        for word in words_to_remove {
            let start = reference_text.find(word).unwrap();
            let end = start + word.len();
            reference_text.replace_range(start..end, "");

            text._delete_word(start, end);
            assert_eq!(text.text, reference_text.chars().collect::<Vec<char>>())
        }
    }

    #[test]
    fn test_swap_words() {
        let orig_text = "This is a semi-short test string (that'll test some edge cases).";
        let mut text = AlteredText::new(orig_text.chars().collect());
        let words_to_swap = [
            ("semi-short", "cases"),
            ("This", "that'll"),
            ("string", "string"),
        ];

        let mut reference_text = orig_text.to_string();
        for (l, r) in words_to_swap {
            let start_l = reference_text.find(l).unwrap();
            let end_l = start_l + l.len();
            let l_copy = reference_text[start_l..end_l].to_string();

            let start_r = reference_text.find(r).unwrap();
            let end_r = start_r + r.len();
            let r_copy = reference_text[start_r..end_r].to_string();

            reference_text.replace_range(start_l..end_l, &r_copy);

            // String indices may have changed, need to re-find the right string.
            let size_diff = (end_r - start_r) as i64 - (end_l - start_l) as i64;
            let new_start_r: usize = (start_r as i64 + size_diff).try_into().unwrap();
            let new_end_r: usize = (end_r as i64 + size_diff).try_into().unwrap();
            reference_text.replace_range(new_start_r..new_end_r, &l_copy);

            text._swap_words(start_l, end_l, start_r, end_r);
            assert_eq!(text.text, reference_text.chars().collect::<Vec<char>>())
        }
    }

    #[test]
    fn test_substitute_words() {
        let orig_text = "This is a semi-short test string (that'll test some edge cases).";
        let mut text = AlteredText::new(orig_text.chars().collect());
        let words_to_substitute = [
            ("semi-short", "cases"),
            ("This", "that'll"),
            ("string", "string"),
        ];

        let mut reference_text = orig_text.to_string();
        for (l, r) in words_to_substitute {
            let start_l = reference_text.find(l).unwrap();
            let end_l = start_l + l.len();
            let l_copy = reference_text[start_l..end_l].to_string();

            let start_r = reference_text.find(r).unwrap();
            let end_r = start_r + r.len();

            reference_text.replace_range(start_r..end_r, &l_copy);

            text._substitute_words(start_l, end_l, start_r, end_r);
            assert_eq!(text.text, reference_text.chars().collect::<Vec<char>>())
        }
    }

    #[test]
    fn test_replace_chars() {
        let orig_text = "This is a semi-short test string (that'll test some edge cases).";
        let mut text = AlteredText::new(orig_text.chars().collect());
        let chars_to_replace = [
            ('s', 'x', 8, 2),
            ('-', 'w', 20, 5),
            ('i', 'i', 0, 5),
            ('s', 'q', 61, 2),
        ];

        let mut reference_text = orig_text.to_string();
        for (target, source, start, count) in chars_to_replace {
            let right_max_replacements = reference_text[start..].matches(target).count();
            let tmp_right = reference_text[start..].replacen(&target.to_string(), &source.to_string(), count);
            reference_text.replace_range(start.., &tmp_right);

            if right_max_replacements < count {
                // Still have more replacements to wrap around
                let tmp_left = reference_text[..start].replacen(&target.to_string(), &source.to_string(), count - right_max_replacements);
                reference_text.replace_range(..start, &tmp_left);
            }

            text._replace_chars(target, source, start, count);
            assert_eq!(text.text, reference_text.chars().collect::<Vec<char>>())
        }
    }

    #[test]
    fn test_delete_chars() {
        let orig_text = "This is a semi-short test string (that'll test some edge cases).";
        let mut text = AlteredText::new(orig_text.chars().collect());
        let chars_to_delete = [
            ('s', 8, 2),
            ('-', 20, 5),
            ('i', 0, 5),
            ('s', 54, 2),
        ];

        let mut reference_text = orig_text.to_string();
        for (target, start, count) in chars_to_delete {
            let mut deleted = 0;

            let right_string: String = reference_text[start..].to_string();
            for (i, _) in right_string.match_indices(target) {
                if deleted >= count {
                    break;
                }
                reference_text.remove(start + i - deleted);
                deleted += 1;
            }

            let deleted_right = deleted;

            // Wrap around and delete from beginning if still needed
            if deleted < count {
                let left_string: String = reference_text[..start].to_string();
                for (j, _) in left_string.match_indices(target) {
                    if deleted >= count {
                        break;
                    }
                    reference_text.remove(j - (deleted - deleted_right));
                    deleted += 1;
                }
            }

            text._delete_chars(target, start, count);
            assert_eq!(text.text, reference_text.chars().collect::<Vec<char>>())
        }
    }
}

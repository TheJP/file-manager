pub struct MatchResult {
    score: u16,
    target: String,
    matches: Vec<usize>,
}

impl MatchResult {
    pub fn score(&self) -> u16 {
        self.score
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    pub fn matches(&self) -> &[usize] {
        &self.matches
    }
}

/// Compares the two given strings by searching for search in target.
///
/// The comparison is case-insensitive and search does not have to be
/// contained in target contiguously but characters have to appear in
/// the right order. Every character of search has to be contained in
/// target to be considered a match.
///
/// A score is computed which is higher for longer contiguous sections
/// of search in target. So for example `compare("oo", "boot")` results
/// in a higher score than `compare("oo", "ovo")`.
pub fn compare(search: &str, target: &str) -> Option<MatchResult> {
    const MAX_SIZE: usize = 1000;

    if search.is_empty() || target.is_empty() {
        return None;
    }

    let original_target = target;
    let search: Vec<_> = search.chars().flat_map(|c| c.to_lowercase()).collect();
    let target: Vec<_> = target.chars().flat_map(|c| c.to_lowercase()).collect();

    if search.len() > MAX_SIZE || target.len() > MAX_SIZE {
        return None;
    }

    if !fast_check(&search, &target) {
        return None;
    }

    debug_assert!(2 * MAX_SIZE < u16::MAX as usize);
    let height = search.len() + 1;
    let width = target.len() + 1;
    let mut scores = vec![0u16; height * width];
    let mut contiguous = scores.clone();

    // Find best match in O(n*m) using dynamic programming.
    for (search_index, search_char) in search.iter().enumerate() {
        for (target_index, target_char) in target.iter().enumerate() {
            let index = search_index * width + target_index;
            let score = (search_char != target_char)
                .then_some(0)
                .unwrap_or_else(|| scores[index] + 1 + contiguous[index]);

            let index_below = (search_index + 1) * width + target_index;
            let index_diagonal = index_below + 1;

            (scores[index_diagonal], contiguous[index_diagonal]) = if score < scores[index_below] {
                (scores[index_below], 0)
            } else if scores[index] == 0 && search_index > 0 {
                (0, 0)
            } else {
                (score, contiguous[index] + 1)
            }
        }
    }

    let score = *scores.last().unwrap();
    if score == 0 {
        return None;
    }

    // Collect match positions for returned result in O(n+m)
    let mut matches = vec![0; search.len()];
    let mut target_index = target.len();
    for search_index in (1..=search.len()).rev() {
        let mut index = search_index * width + target_index;
        while contiguous[index] == 0
            || (target_index > 1
                && scores[index - 1] == scores[index]
                && (search_index == search.len() || matches[search_index] != target_index))
        {
            target_index -= 1;
            index -= 1;
        }
        target_index -= 1;
        matches[search_index - 1] = target_index;
    }

    // Use original (mixed-case) characters for the output if possible.
    let target = if target.len() != original_target.chars().count() {
        target.into_iter().collect()
    } else {
        original_target.to_string()
    };

    Some(MatchResult {
        score,
        target,
        matches,
    })
}

/// Checks that search is contained non-contiguously in target in O(n+m).
#[inline]
fn fast_check(search: &[char], target: &[char]) -> bool {
    if search.len() > target.len() {
        return false;
    }

    let mut target_index = 0;
    for &search_char in search {
        while target_index < target.len() && target[target_index] != search_char {
            target_index += 1;
        }
        if target_index >= target.len() {
            return false;
        }
        target_index += 1;
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_check_simple() {
        assert!(fast_check(&['b'], &['a', 'b', 'b', 'c']));
        assert!(fast_check(&['a', 'b', 'b'], &['a', 'b', 'b', 'c']));
        assert!(fast_check(&['a', 'c'], &['a', 'b', 'b', 'c']));
        assert!(!fast_check(&['a', 'd'], &['a', 'b', 'b', 'c']));
        assert!(!fast_check(&['a', 'a', 'a'], &['a', 'b', 'b', 'c']));
    }

    #[test]
    fn fast_check_case_sensitive() {
        assert!(!fast_check(&['B'], &['a', 'b', 'b', 'c']));
        assert!(fast_check(&['a', 'b', 'C'], &['a', 'b', 'b', 'C']));
        assert!(!fast_check(&['a', 'b', 'C'], &['a', 'b', 'b', 'c']));
    }

    #[test]
    fn fast_check_unicode() {
        assert!(fast_check(&['a', 'c'], &['a', '💚', '💚', 'c']));
        assert!(fast_check(&['💚'], &['a', '💚', '💚', 'c']));
        assert!(fast_check(&['💚', 'c'], &['a', '💚', '💚', 'c']));
        assert!(!fast_check(&['❤'], &['a', '💚', '💚', 'c']));
    }

    #[test]
    fn compare_simple_edge_cases() {
        assert!(compare("", "").is_none());
        assert!(compare("abc", "").is_none());
        assert!(compare("", "abc").is_none());
        assert!(compare("", "abc").is_none());
        assert!(compare("abcd", "abc").is_none());
    }

    #[test]
    fn compare_simple_contiguous() {
        let better = compare("xy", "xy").unwrap().score();
        let worse = compare("xy", "somex somey").unwrap().score();
        assert!(better > worse, "should score contiguous match higher");
    }

    #[test]
    fn compare_simple_ordering() {
        let search = "commIT";
        let mut results: Vec<_> = [
            compare(search, "nonsense"),    // No Match
            compare(search, "c_o_m_m_I_T"), // All Spaced
            compare(search, "c_ommIT"),     // One Space
            compare(search, "commIT"),      // Exact Match
        ]
        .into_iter()
        .flatten()
        .map(|result| result.score())
        .collect();

        assert_eq!(3, results.len());
        let expected = results.clone();
        results.sort();
        assert_eq!(expected, results);
    }

    #[test]
    fn compare_no_match() {
        assert!(compare("nonsense", "something else").is_none());
        assert!(compare("commit", "comm nonesense").is_none());
        assert!(compare("commit", "comm nonesense t").is_none());
        assert!(compare("commit", "short").is_none());
    }

    #[test]
    fn compare_case_insensitive() {
        assert!(compare("commIT", "COMMit").is_some());
        assert!(compare("COMMit", "com MIT").is_some());
    }

    #[test]
    fn compare_matches() {
        let search = "commit";

        let result = compare(search, "commit").unwrap();
        assert!(result.matches.into_iter().eq(0..search.len()));

        let result = compare(search, "something commit").unwrap();
        assert_eq!(&[10, 11, 12, 13, 14, 15], result.matches());

        let result = compare(search, "c_om_##_mit").unwrap();
        assert_eq!(&[0, 2, 3, 8, 9, 10], result.matches());
    }

    #[test]
    fn compare_finds_first_match() {
        let search = "commit";

        let result = compare(search, "commit commit").unwrap();
        assert!(result.matches.into_iter().eq(0..search.len()));

        let result = compare(search, "something commit commit").unwrap();
        assert_eq!(&[10, 11, 12, 13, 14, 15], result.matches());

        let result = compare("xy", "somex somexy").unwrap();
        assert_eq!(&[10, 11], result.matches());
    }

    #[test]
    fn compare_finds_non_contiguous_matches() {
        let result = compare("xy", "somex somey").unwrap();
        assert_eq!(&[4, 10], result.matches());
    }

    #[test]
    fn compare_returned_target_string() {
        let result = compare("commit", "commit commit").unwrap();
        assert_eq!("commit commit", result.target());

        let result = compare("something", "commit SomeThing commit").unwrap();
        assert_eq!("commit SomeThing commit", result.target());

        let result = compare("something", "İ SomeThing İ").unwrap();
        assert_eq!("i\u{0307} something i\u{0307}", result.target());
        assert_eq!(&[3, 4, 5, 6, 7, 8, 9, 10, 11], result.matches());
    }

    #[test]
    fn compare_matches_unicode() {
        let result = compare("💚💚", "some💚 some💚");
        assert!(result.is_some());
        assert_eq!(&[4, 10], result.unwrap().matches());
    }
}

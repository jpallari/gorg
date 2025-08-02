const WORD_SEPARATORS: &[char] = &[' ', '-', ',', '.', '_', '/', '\\'];

pub struct Keywords<'a> {
    parts: Vec<&'a str>,
}

impl<'a> Default for Keywords<'a> {
    fn default() -> Self {
        Self { parts: Vec::new() }
    }
}

impl<'a> Keywords<'a> {
    pub fn new(data: &'a str) -> Self {
        let parts: Vec<&'a str> = data
            .split(WORD_SEPARATORS)
            .filter(|p| !p.is_empty())
            .collect();
        Self { parts }
    }

    pub fn is_empty(&self) -> bool {
        self.parts.is_empty()
    }

    pub fn score(&self, target: &str) -> f32 {
        let mut score: f32 = 0.;
        for (pi, p) in self.parts.iter().enumerate() {
            let mut part_score: f32 = 0.;
            for (ti, t) in target
                .split(WORD_SEPARATORS)
                .filter(|p| !p.is_empty())
                .enumerate()
            {
                part_score += t
                    .match_indices(p)
                    .next()
                    .map(|(i, _)| {
                        let distance = match ti.max(pi) - ti.min(pi) {
                            0 => 1.,
                            1 => 0.9,
                            2 => 0.8,
                            3 => 0.7,
                            _ => 0.6,
                        };
                        let filled = p.len() as f32 / t.len() as f32;
                        let index = 1. - (i as f32 / t.len() as f32);
                        filled * 2. + index * 2. * distance
                    })
                    .unwrap_or(0.)
            }
            if part_score == 0. {
                // If the part does not match any of the target parts,
                // we know that we can't have a proper match for the keywords.
                return 0.;
            }
            score += part_score;
        }
        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_no_match() {
        let score = Keywords::new("foo").score("github.com/jpallari/gorg");
        assert_eq!(score, 0.);
    }

    #[test]
    fn score_partial_match() {
        let score = Keywords::new("jp foo").score("github.com/jpallari/gorg");
        assert_eq!(score, 0.);
    }

    #[test]
    fn score_full_match_1() {
        let score = Keywords::new("g jp go").score("github.com/jpallari/gorg");
        assert!(score > 0., "{score} > 0");
    }

    #[test]
    fn score_full_match_2() {
        let score = Keywords::new("gi jp").score("github.com/jpallari/gorg");
        assert!(score > 0., "{score} > 0");
    }

    #[test]
    fn score_comparative() {
        let kws = Keywords::new("go");
        let score1 = kws.score("github.com/golang/go");
        let score2 = kws.score("github.com/jpallari/go");
        let score3 = kws.score("github.com/jpallari/gorg");
        let score4 = kws.score("github.com/jpallari/hugo");
        assert!(score1 > score3, "{score1} > {score3}");
        assert!(score2 > score3, "{score2} > {score3}");
        assert!(score3 > score4, "{score3} > {score4}");
    }
}

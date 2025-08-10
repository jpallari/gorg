use crate::text;

pub fn calc_score(matcher: &str, target: &str) -> f32 {
    let mut score: f32 = 0.;

    for (pi, p) in matcher
        .split(text::is_punctuation)
        .filter(|p| !p.trim().is_empty())
        .enumerate()
    {
        let mut part_score: f32 = 0.;
        for (ti, t) in target
            .split(text::is_punctuation)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_no_match() {
        let score = calc_score("foo", "github.com/jpallari/gorg");
        assert_eq!(score, 0.);
    }

    #[test]
    fn score_partial_match() {
        let score = calc_score("jp foo", "github.com/jpallari/gorg");
        assert_eq!(score, 0.);
    }

    #[test]
    fn score_full_match_1() {
        let score = calc_score("g jp go", "github.com/jpallari/gorg");
        assert!(score > 0., "{score} > 0");
    }

    #[test]
    fn score_full_match_2() {
        let score = calc_score("gi jp", "github.com/jpallari/gorg");
        assert!(score > 0., "{score} > 0");
    }

    #[test]
    fn score_comparative() {
        let matcher = "go";
        let score1 = calc_score(matcher, "github.com/golang/go");
        let score2 = calc_score(matcher, "github.com/jpallari/go");
        let score3 = calc_score(matcher, "github.com/jpallari/gorg");
        let score4 = calc_score(matcher, "github.com/jpallari/hugo");
        assert!(score1 > score3, "{score1} > {score3}");
        assert!(score2 > score3, "{score2} > {score3}");
        assert!(score3 > score4, "{score3} > {score4}");
    }
}

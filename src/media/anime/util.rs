use crate::Locale;

#[cfg(feature = "experimental-stabilizations")]
pub(crate) fn parse_locale_from_slug_title<S: AsRef<str>>(slug_title: S) -> Locale {
    split_locale_from_slug_title(slug_title).1
}

#[cfg(feature = "experimental-stabilizations")]
pub(crate) fn split_locale_from_slug_title<S: AsRef<str>>(slug_title: S) -> (String, Locale) {
    let title = slug_title.as_ref().trim_end_matches("-dub").to_string();

    let locales = vec![
        ("-arabic", Locale::ar_SA),
        ("-castilian", Locale::es_ES),
        ("-english", Locale::en_US),
        ("-english-in", Locale::en_IN),
        ("-french", Locale::fr_FR),
        ("-german", Locale::de_DE),
        ("-hindi", Locale::hi_IN),
        ("-italian", Locale::it_IT),
        ("-portuguese", Locale::pt_BR),
        ("-russian", Locale::ru_RU),
        ("-spanish", Locale::es_419),
        ("-japanese-audio", Locale::ja_JP),
    ];
    for (end, locale) in locales {
        if title.ends_with(end) {
            return (title.trim_end_matches(end).to_string(), locale);
        }
    }
    (title, Locale::ja_JP)
}

/// Remove all duplicates from a [`Vec`].
pub(crate) fn real_dedup_vec<T: Clone + Eq>(input: &mut Vec<T>) {
    let mut dedup = vec![];
    for item in input.clone() {
        if !dedup.contains(&item) {
            dedup.push(item);
        }
    }
    *input = dedup
}

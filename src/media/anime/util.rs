#[cfg(feature = "experimental-stabilizations")]
pub(crate) fn parse_locale_from_slug_title<S: AsRef<str>>(slug_title: S) -> crate::Locale {
    split_locale_from_slug_title(slug_title).1
}

#[cfg(feature = "experimental-stabilizations")]
pub(crate) fn split_locale_from_slug_title<S: AsRef<str>>(
    slug_title: S,
) -> (String, crate::Locale) {
    let title = slug_title.as_ref().trim_end_matches("-dub").to_string();

    let locales = vec![
        ("-arabic", crate::Locale::ar_SA),
        ("-castilian", crate::Locale::es_ES),
        ("-english", crate::Locale::en_US),
        ("-english-in", crate::Locale::en_IN),
        ("-french", crate::Locale::fr_FR),
        ("-german", crate::Locale::de_DE),
        ("-hindi", crate::Locale::hi_IN),
        ("-italian", crate::Locale::it_IT),
        ("-portuguese", crate::Locale::pt_BR),
        ("-russian", crate::Locale::ru_RU),
        ("-spanish", crate::Locale::es_419),
        ("-japanese-audio", crate::Locale::ja_JP),
    ];
    for (end, locale) in locales {
        if title.ends_with(end) {
            return (title.trim_end_matches(end).to_string(), locale);
        }
    }
    (title, crate::Locale::ja_JP)
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

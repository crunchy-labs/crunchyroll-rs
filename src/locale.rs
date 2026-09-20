//! Locales of the languages which Crunchyroll supports.

use crate::enum_values;

enum_values! {
    /// Enum of supported languages by Crunchyroll.
    /// Crunchyroll lists the available languages in the following api results:
    /// - <https://static.crunchyroll.com/config/i18n/v3/audio_languages.json>
    /// - <https://static.crunchyroll.com/config/i18n/v3/timed_text_languages.json>
    #[allow(non_camel_case_types)]
    #[derive(Hash, Ord, PartialOrd)]
    pub enum Locale {
        ar_SA = "ar-SA"
        ca_ES = "ca-ES"
        de_DE = "de-DE"
        en_IN = "en-IN"
        en_US = "en-US"
        es_419 = "es-419"
        es_ES = "es-ES"
        fr_FR = "fr-FR"
        hi_IN = "hi-IN"
        id_ID = "id-ID"
        it_IT = "it-IT"
        ja_JP = "ja-JP"
        ko_KR = "ko-KR"
        ms_MY = "ms-MY"
        pl_PL = "pl-PL"
        pt_BR = "pt-BR"
        pt_PT = "pt-PT"
        ru_RU = "ru-RU"
        ta_IN = "ta-IN"
        te_IN = "te-IN"
        th_TH = "th-TH"
        tl_PH = "tl-PH"
        tr_TR = "tr-TR"
        vi_VN = "vi-VN"
        zh_CN = "zh-CN"
        zh_HK = "zh-HK"
        zh_TW = "zh-TW"
    }
}

impl Locale {
    /// All available locales.
    pub const fn all() -> &'static [Locale] {
        &[
            Locale::ar_SA,
            Locale::ca_ES,
            Locale::de_DE,
            Locale::en_IN,
            Locale::en_US,
            Locale::es_419,
            Locale::es_ES,
            Locale::fr_FR,
            Locale::hi_IN,
            Locale::id_ID,
            Locale::it_IT,
            Locale::ja_JP,
            Locale::ko_KR,
            Locale::ms_MY,
            Locale::pl_PL,
            Locale::pt_BR,
            Locale::pt_PT,
            Locale::ru_RU,
            Locale::ta_IN,
            Locale::te_IN,
            Locale::th_TH,
            Locale::tl_PH,
            Locale::tr_TR,
            Locale::vi_VN,
            Locale::zh_CN,
            Locale::zh_CN,
            Locale::zh_TW,
        ]
    }

    /// Converts the locale into a (english) human-readable string.
    pub fn to_human_readable(&self) -> &str {
        match self {
            Locale::ar_SA => "Arabic (Saudi Arabia)",
            Locale::ca_ES => "Catalan",
            Locale::de_DE => "German",
            Locale::en_IN => "English (India)",
            Locale::en_US => "English (US)",
            Locale::es_419 => "Spanish (Latin America)",
            Locale::es_ES => "Spanish (Spain)",
            Locale::fr_FR => "French",
            Locale::hi_IN => "Hindi",
            Locale::id_ID => "Indonesian",
            Locale::it_IT => "Italian",
            Locale::ja_JP => "Japanese",
            Locale::ko_KR => "Korean",
            Locale::ms_MY => "Malay",
            Locale::pl_PL => "Polish",
            Locale::pt_BR => "Portuguese (Brazil)",
            Locale::pt_PT => "Portuguese (Portugal)",
            Locale::ru_RU => "Russian",
            Locale::ta_IN => "Tamil",
            Locale::te_IN => "Telugu",
            Locale::th_TH => "Thai",
            Locale::tl_PH => "Filipino",
            Locale::tr_TR => "Turkish",
            Locale::vi_VN => "Vietnamese",
            Locale::zh_CN => "Chinese (China)",
            Locale::zh_HK => "Chinese (Cantonese)",
            Locale::zh_TW => "Chinese (Mandarin)",
            Locale::Custom(custom) => custom.as_str(),
        }
    }
}

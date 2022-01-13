use crate::translations::GearBotLangKey;
use fluent_bundle::{bundle::FluentBundle as RawBundle, FluentArgs, FluentMessage, FluentResource, FluentValue};
use intl_memoizer::concurrent::IntlLangMemoizer;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};
use unic_langid::LanguageIdentifier;

const FAILED_TRANSLATE_FALLBACK_MSG: &str =
    "Translation failure occurred: unable to localise '{}'. This is a bug and has been logged as such.";

type FluentBundle = RawBundle<FluentResource, IntlLangMemoizer>;

pub struct Translator {
    translations: HashMap<String, Arc<FluentBundle>>,
    master_lang: String,
}

pub struct MessageTranslator<'a> {
    key: GearBotLangKey,
    bundle: Arc<FluentBundle>,
    message: Option<FluentMessage<'a>>,
    args: Option<FluentArgs<'a>>,
}

impl Translator {
    // Construct a new translator by loading the specified directory
    // This directory should have a folder per language, named by its identifier.
    // All files in this directory will be loaded into the lang bundle for the language.
    pub fn new(lang_dir: &str, master_lang: String) -> Translator {
        info!("Loading translations...");

        let translation_dir =
            fs::read_dir(lang_dir).unwrap_or_else(|_| panic!("Unable to read translations directory '{}'", lang_dir));

        let mut translations = HashMap::new();

        // Check all child dirs
        for child_result in translation_dir {
            let child = child_result.expect("Failed to get directory metadata");
            let dir_name = child.file_name().to_string_lossy().to_string();
            if !child
                .file_type()
                .unwrap_or_else(|_| panic!("Unable to determine filetype of '{}'", dir_name))
                .is_dir()
            {
                warn!("Ignoring '{}' as it's not a directory", dir_name);
                continue;
            }

            // make sure the identifier is valid
            if let Ok(identifier) = dir_name.parse::<LanguageIdentifier>() {
                debug!("Loading translations for {}", dir_name);
                let langs = vec![identifier];
                let mut bundle = FluentBundle::new_concurrent(langs);
                // bundle.set_use_isolating(false);

                // read and combine all files in the directory
                let lang_dir =
                    fs::read_dir(child.path()).unwrap_or_else(|_| panic!("Unable to read lang dir '{}'", dir_name));
                for file_result in lang_dir {
                    let file = file_result.expect("Failed to get file metadata from dir");
                    let file_name = file.file_name().to_string_lossy().to_string();
                    trace!("Loading file {}", file_name);
                    let file_content = match File::open(file.path()) {
                        Ok(content) => content,
                        Err(e) => {
                            error!("Failed to open file {} for lang {}: {}", file_name, dir_name, e);
                            continue;
                        }
                    };
                    let reader = BufReader::new(file_content).lines();

                    for line_result in reader {
                        let line = line_result.expect("Failed to read line");
                        let resource = FluentResource::try_new(line.clone());

                        match resource {
                            Ok(resource) => {
                                bundle
                                    .add_resource(resource)
                                    .unwrap_or_else(|_| panic!("Failed to add entry to the bundle: {}", &line));
                            }
                            Err(e) => {
                                error!(
                                    "Corrupt entry encountered in file {} from lang {}: {:?} ({})",
                                    file_name, dir_name, e.1, line
                                );
                            }
                        }
                    }
                }

                translations.insert(dir_name.to_string(), Arc::new(bundle));
            } else {
                warn!("Ignoring '{}' as it's not a valid language identifier", dir_name);
            }
        }

        info!("Successfully loaded {} languages", translations.len());

        if !translations.contains_key(&master_lang) {
            panic!(
                "{} was designated as master language, but no translations where provided for this language!",
                master_lang
            )
        }

        Translator {
            translations,
            master_lang,
        }
    }

    // New translator
    pub fn translate(&self, lang: &str, key: GearBotLangKey) -> MessageTranslator {
        let translation_key = key.as_str();
        let (translations, lang) = if let Some(translations) = self.translations.get(lang) {
            (translations, lang)
        } else {
            debug!(
                "Attempted to translate to unknown lang {}, falling back to {}",
                lang, self.master_lang
            );
            //safe to unwrap, we ensured this is present during initialization
            (
                self.translations.get(&self.master_lang).unwrap(),
                self.master_lang.as_str(),
            )
        };
        let mut message = translations.get_message(translation_key);

        // not found, try the master language if we where not already using that
        if message.is_none() && lang != self.master_lang {
            message = self
                .translations
                .get(&self.master_lang)
                .unwrap()
                .get_message(translation_key);
        }

        MessageTranslator {
            key,
            bundle: translations.clone(),
            message,
            args: Default::default(),
        }
    }
}

impl<'a> MessageTranslator<'a> {
    pub fn arg<P>(mut self, key: &'a str, value: P) -> Self
    where
        P: Into<FluentValue<'a>>,
    {
        let mut args = match self.args {
            None => FluentArgs::new(),
            Some(args) => args,
        };
        args.set(key, value.into());
        self.args = Some(args);
        self
    }

    pub fn build(&self) -> Cow<str> {
        let mut errors = Vec::new();

        match &self.message {
            None => {
                error!("Tried to translate non existing lang key: {}", self.key.as_str());
                Cow::Borrowed(FAILED_TRANSLATE_FALLBACK_MSG)
            }
            Some(message) => {
                let translated = self
                    .bundle
                    .format_pattern(message.value().unwrap(), self.args.as_ref(), &mut errors);

                if errors.is_empty() {
                    translated
                } else {
                    error!(
                        "Translation failure(s) when translating {} with args {:?}: {:?}",
                        self.key.as_str(),
                        self.args,
                        errors
                    );
                    Cow::Borrowed(FAILED_TRANSLATE_FALLBACK_MSG)
                }
            }
        }
    }
}

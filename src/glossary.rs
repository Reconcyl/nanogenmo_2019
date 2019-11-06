use std::collections::HashMap;

use super::{WordArena, AnnotatedString, Word};

pub(super) type Glossary = HashMap<Word, Option<AnnotatedString>>;

pub const RANDOM_SIGNAL: &str = "::::";

/// Words that are defined in the global glossary.
const DEFINED: &[(&str, &str)] = &[
    ("glossary",      "The part of a book with definitions of words used in it."),
    ("words",         "See 'word.'"),
    ("word",          "You're reading them."),
    ("definitions",   "See 'definition.'"),
    ("definition",    "You're reading one."),
    ("book",          "You're reading one."),
    ("reading",       "You're doing it."),
    ("index",         "The part of a book that indicates where words are used."),
    ("indicates",     "See 'indicate.'"),
    ("indicate",      "I'm doing it."),
    ("time",          "You're going through it."),
    ("end",           "I doubt you'll get there."),
    ("table",         "An organized list."),
    ("list",          "Nothing, or cons."),
    ("cons",          "Something, and a list."),
    ("organized",     "I wish I knew."),
    ("contents",      "What's in."),
    ("chapter",       "The central component of a book."),
    ("central",       "<insert in-joke>"),
    ("joke",          "What's made more confusing by being read out-of-order?"),
    ("confusing",     "See this book."),
    ("read",          "What you're supposed to do to a book, but shouldn't to this one."),
    ("afterword",     "Words which are said after."),
    ("figures",       "People, numbers, and drawings."),
    ("people",        "Dumb, panicky dangerous animals and you know it."),
    ("numbers",       "See list of figures for examples."),
    ("drawings",      "The best kind of math."),
    ("examples",      "See 'example.'"),
    ("example",       "See 'examples' for an example."),
    ("math",          "See 'numbers.'"),
    ("you",           "You're being it."),
    ("know",          "Getting philosophical, are we?"),
    ("philosophical", "Silly and distracting in nature."),
    ("fourwords",     "Four words, written forwards, comprising forewords."),
    ("fourword",      "See 'fourwords.'"),
    ("four",          "See 'three,' then add one."),
    ("three",         "See 'two,' then add one."),
    ("two",           "See 'one,' then add one."),
    ("add",           "See 'math.'"),
    ("one",           "Something."),
    ("forwards",      "Not left. Not right."),
    ("dedicated",     "Given a dedication."),
    ("dedication",    "What one would need to write all these definitions."),
    ("nanogenmo",     "It's this programming event - maybe you've heard of it?"),
    ("programming",   "I'm doing it."),
    ("maybe",         "Something or nothing."),
    ("exception",     "It's supposed to mean something."),
    ("random",        RANDOM_SIGNAL),
    ("generated",     "See 'generation.'"),
    ("generates",     "See 'generation.'"),
    ("generation",    "It's happening."),
    ("known",         "See 'know.'"),
    ("theory",        "See 'math.'"),
    ("issue",         "See my code for examples."),
    ("code",          "You're why it exists."),
    ("think",         "We all need to do it."),
    ("look",          "You're doing it."),
    ("probability",   "See 'math.'"),
    ("find",          "You've done it!"),
    ("suggestion",    "We all have them."),
];
/// Words that are left undefined.
const UNDEFINED: &[&str] = &["the", "part", "of", "a", "that", "used", "in", "it", "you're", "doing", "where", "see", "with", "them", "i'm", "are", "not", "given", "this", "is", "once", "upon", "going", "through", "i", "get", "there", "doubt", "you'll", "an", "wish", "knew", "nil", "something", "nothing", "or", "and", "what's", "insert", "component", "here", "academia", "made", "more", "by", "being", "what", "out", "supposed", "to", "order", "do", "but", "shouldn't", "which", "said", "after", "dumb", "panicky", "best", "for", "dangerous", "animals", "kind", "getting", "silly", "distracting", "nature", "we", "written", "then", "comprising", "forewords", "left", "right", "all", "material", "hello", "following", "would", "need", "write", "these", "lucky", "your", "p", "s", "fun", "else", "really", "about", "ids", "was", "shared", "precisely", "than", "higher", "id", "section", "sections", "recommended", "community", "it's", "event", "you've", "heard", "mean", "when", "trust", "accuracy", "happening", "been", "just", "could", "entire", "purpose", "did", "chose", "happened", "my", "exists", "why", "submitted", "isn't", "low", "pretty", "rs", "main", "line", "wait", "can", "on", "appear", "message", "source", "book's", "into", "go", "have", "done", "playing", "they're", "character", "narrator", "reconcyl", "author", "text", "reader", "dear"];

pub(super) fn get_global_glossary(arena: &mut WordArena) -> Glossary {
    let mut glossary = HashMap::new();
    for (term, def) in DEFINED {
        let term = arena.get(term);
        let def = AnnotatedString::new(arena, def.to_string());
        assert!(glossary.insert(term, Some(def)).is_none());
    }
    for term in UNDEFINED {
        let term = arena.get(term);
        assert!(glossary.insert(term, None).is_none());
    }
    // Make sure that the glossary is closed (it never uses a word without explicitly defining or not defining it)
    for word in glossary.values().filter_map(Option::as_ref).flat_map(|def| def.words.iter()) {
        if !glossary.contains_key(word) {
            panic!("'{}' is not defined", arena.name(*word));
        }
    }
    glossary
}
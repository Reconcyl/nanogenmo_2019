use lazy_static::lazy_static;
use regex::Regex;
use rand::Rng;
use id_arena::ArenaBehavior as _;

use std::collections::{HashMap, BTreeMap, BTreeSet, VecDeque};
use std::fmt::{self, Display};

mod glossary;
use glossary::Glossary;

type Word = id_arena::Id<String>;
type SectionId = u16;

#[derive(Debug)]
struct WordArena {
    arena: id_arena::Arena<String>,
    arena_id: Option<u32>,
    mapping: HashMap<String, Word>,
}

impl WordArena {
    fn new() -> Self {
        Self {
            arena: id_arena::Arena::new(),
            arena_id: None,
            mapping: HashMap::new(),
        }
    }
    fn get(&mut self, word: &str) -> Word {
        assert!(inflections::case::is_lower_case(word));
        self.mapping.get(word).copied()
            .unwrap_or_else(|| {
                let word = word.to_string();
                let id = self.arena.alloc(word.clone());
                self.arena_id = Some(id_arena::DefaultArenaBehavior::arena_id(id));
                self.mapping.insert(word, id);
                id
            })
    }
    fn name(&self, id: Word) -> &str {
        self.arena.get(id).unwrap()
    }
    fn pick_random(&self) -> &str {
        let idx = rand::thread_rng().gen_range(0, self.arena.len());
        self.name(id_arena::DefaultArenaBehavior::new_id(self.arena_id.unwrap(), idx))
    }
}

#[derive(Debug)]
struct AnnotatedString {
    content: String,
    words: Vec<Word>,
}

impl Display for AnnotatedString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.content, f)
    }
}

impl AnnotatedString {
    /// Find all the words in a string.
    fn new(arena: &mut WordArena, content: String) -> Self {
        lazy_static! {
            static ref WORD_REGEX: Regex = Regex::new(r"[A-Za-z]+(?:'[A-Za-z]+)*").unwrap();
        }
        let words = WORD_REGEX.find_iter(&content)
            .map(|m| arena.get(&m.as_str().to_lowercase()))
            .collect();
        Self { content, words }
    }
    /// Get the word count.
    fn word_count(&self) -> usize {
        self.words.len()
    }
}

struct IdGenerator(BTreeSet<SectionId>);

impl IdGenerator {
    fn gen(&mut self) -> SectionId {
        loop {
            let id = rand::random();
            if !self.0.contains(&id) {
                self.0.insert(id);
                return id;
            }
        }
    }
}

#[derive(Clone, Copy)]
enum SectionType {
    Dedication,
    Fourword,
    TableOfContents,
    Chapter1,
    Glossary,
    ListOfFigures,
    Index,
    Afterword,
}

struct Section {
    content: AnnotatedString,
    id: SectionId,
    type_: SectionType,
}

impl Section {
    fn with_id(idg: &mut IdGenerator, type_: SectionType, f: impl FnOnce(SectionId) -> AnnotatedString) -> Self {
        let id = idg.gen();
        Self { content: f(id), id, type_ }
    }
    fn word_count(&self) -> usize {
        self.content.word_count()
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.content, f)
    }
}

fn render_chapter_1(idg: &mut IdGenerator, arena: &mut WordArena) -> Section {
    Section::with_id(idg, SectionType::Chapter1, |id| {
        let rendered = format!("## Chapter 1 (#{})\n\n\\<Insert academia joke here>", id);
        AnnotatedString::new(arena, rendered)
    })
}

fn render_dedication(idg: &mut IdGenerator, arena: &mut WordArena) -> Section {
    Section::with_id(idg, SectionType::Dedication, |id| {
        let rendered = format!("## Dedication (#{0})\n\n\
            All material following this dedication is dedicated to the NaNoGenMo 2019 community, \
            with the exception of sections with an ID higher than this one (#{0}).", id);
        AnnotatedString::new(arena, rendered)
    })
}

fn render_table_of_contents<'a>(idg: &mut IdGenerator, arena: &mut WordArena, sections: impl Iterator<Item=&'a Section>) -> Section {
    Section::with_id(idg, SectionType::TableOfContents, |id| {
        let mut rendered = format!("## Table of Contents (#{})\n", id);
        for section in sections {
            rendered.push_str(&format!("\n- **{}** (#{})", match section.type_ {
                SectionType::Dedication => "Dedication",
                SectionType::Fourword => "Fourword",
                SectionType::TableOfContents => "Table of Contents",
                SectionType::Chapter1 => "Chapter 1",
                SectionType::Glossary => "Glossary",
                SectionType::ListOfFigures => "List of Figures",
                SectionType::Index => "Index",
                SectionType::Afterword => "Afterword",
            }, section.id));
        }
        AnnotatedString::new(arena, rendered)
    })
}

fn render_fourword(idg: &mut IdGenerator, arena: &mut WordArena) -> Section {
    Section::with_id(idg, SectionType::Fourword, |id| {
        let mut rendered = format!("## Fourword (#{})\n\n", id);
        // Select four random words from the arena.
        rendered.push_str(&inflections::case::to_title_case(arena.pick_random()));
        rendered.push_str(" ");
        rendered.push_str(arena.pick_random());
        rendered.push_str(" ");
        rendered.push_str(arena.pick_random());
        rendered.push_str(" ");
        rendered.push_str(arena.pick_random());
        rendered.push_str(".");
        AnnotatedString::new(arena, rendered)
    })
}

fn render_glossary<'a>(
    idg: &mut IdGenerator,
    arena: &mut WordArena,
    glossary: &Glossary, 
    sections: impl Iterator<Item=&'a Section>
) -> Section {
    Section::with_id(idg, SectionType::Glossary, |id| {
        let mut words = sections.flat_map(|section| &section.content.words).collect::<Vec<_>>();
        words.sort();
        words.dedup();
        let mut rendered = format!("## Glossary (#{})\n", id);
        for &word in words.into_iter() {
            if !glossary.contains_key(&word) {
                panic!("'{}' is not defined", arena.name(word));
            }
            if let Some(def) = &glossary[&word] {
                rendered.push_str(&format!("\n- **{}** - ", arena.name(word)));
                if def.content == glossary::RANDOM_SIGNAL {
                    rendered.push_str(&format!("See '{}.'", arena.pick_random()));
                } else {
                    rendered.push_str(&def.content);
                }
            }
        }
        AnnotatedString::new(arena, rendered)
    })
}

fn render_list_of_figures<'a>(
    idg: &mut IdGenerator,
    arena: &mut WordArena,
    random_section_id: SectionId,
) -> Section {
    Section::with_id(idg, SectionType::ListOfFigures, |id| {
        let mut rendered = format!("## List of Figures (#{})\n", id);
        let distribution = rand_distr::Normal::new(0.0, 3.0).unwrap();
        let quantity = rand::thread_rng().gen_range(5, 30);
        let mut note = false;
        for _ in 0..quantity {
            rendered.push_str(&format!("\n- {:.3}", rand::thread_rng().sample(distribution)));
            if rand::thread_rng().gen_ratio(1, 10) {
                rendered.push_str(" (*)");
                note = true;
            }
        }
        if note {
            rendered.push_str(&format!(
                "\n\n(*) The accuracy of these numbers is not known. It is recommended not to trust them \
                when reading section #{}.", random_section_id));
        }
        AnnotatedString::new(arena, rendered)
    })
}

fn render_index<'a>(idg: &mut IdGenerator, arena: &mut WordArena, sections: impl Iterator<Item=&'a Section>) -> Section {
    Section::with_id(idg, SectionType::Index, |id| {
        let mut word_uses = BTreeMap::new();
        for section in sections {
            for &word in &section.content.words {
                word_uses.entry(word).or_insert(BTreeSet::new()).insert(section.id);
            }
        }
        let mut rendered = format!("## Index (#{})\n", id);
        for (word, use_set) in word_uses {
            rendered.push_str("\n- **");
            rendered.push_str(arena.name(word));
            rendered.push_str("** - ");
            for (i, id) in use_set.into_iter().enumerate() {
                if i != 0 {
                    rendered.push_str(", ");
                }
                rendered.push_str(&format!("#{}", id));
            }
        }
        AnnotatedString::new(arena, rendered)
    })
}

fn render_afterword<'a>(
    idg: &mut IdGenerator,
    arena: &mut WordArena,
    mut random_section_id: impl FnMut() -> SectionId,
) -> Section {
    Section::with_id(idg, SectionType::Afterword, |id| {
        let mut rendered = format!("## Afterword (#{})\n\n", id);
        if rand::thread_rng().gen_ratio(1, 10_000_000) {
            rendered.push_str(
                "Hello, dear reader! I'm the author of the text you're reading. Not @Reconcyl, but the narrator. The character \
                 they're playing.\n\n\
                 I have a suggestion for you. Go into this book's source code and find the part that generates this message. \
                 What's the probability it would appear? Go on, look. I can wait. It's in `main.rs`, line 268.\n\n\
                 It's pretty low, isn't it? Do you think the one I submitted to the NaNoGenMo issue just happened to have it? \
                 Or do you think Reconcyl chose one that did on purpose?\n\n\
                 This entire book *could*, in theory, have been generated by precisely the code I shared. But *was* it?\n\n\
                 Are the section IDs I used *really* random? What about the fourwords? Or is there something else going on?\n\n\
                 Have fun.\n\n"
            );
            let quantity = rand::thread_rng().gen_range(3, 16);
            for i in 0..quantity {
                rendered.push_str(if i == 0 {
                    "P.S. your lucky section numbers are "
                } else if i == quantity - 1 {
                    ", and "
                } else {
                    ", "
                });
                rendered.push_str(&format!("#{:?}", random_section_id()))
            }
            rendered.push_str(".");
        } else {
            rendered.push_str(&inflections::case::to_title_case(arena.pick_random()));
        }
        AnnotatedString::new(arena, rendered)
    })
}

const GENERATE_TYPES: u8 = 7;
fn generate(word_minimum: usize) -> VecDeque<Section> {
    let mut arena = WordArena::new();
    let glossary = glossary::get_global_glossary(&mut arena);
    let mut sections = VecDeque::new();
    let mut idg = IdGenerator(BTreeSet::new());
    sections.push_back(render_chapter_1(&mut idg, &mut arena));
    let rng_range = rand::distributions::Uniform::from(0..GENERATE_TYPES);
    let random_section_id = |sections: &VecDeque<Section>| {
        let slices = sections.as_slices();
        let idx = rand::thread_rng().gen_range(0, slices.0.len() + slices.1.len());
        if idx < slices.0.len() {
            slices.0[idx].id
        } else {
            slices.1[idx - slices.0.len()].id
        }
    };
    while {
        let words: usize = sections.iter().map(Section::word_count).sum();
        words < word_minimum
    } {
        match rand::thread_rng().sample(rng_range) {
            0 => {
                let section = render_dedication(&mut idg, &mut arena);
                sections.push_front(section);
            }
            1 => {
                let section = render_fourword(&mut idg, &mut arena);
                sections.push_front(section);
            }
            2 => {
                let section = render_table_of_contents(&mut idg, &mut arena, sections.iter());
                sections.push_front(section);
            }
            3 => {
                let section = render_glossary(&mut idg, &mut arena, &glossary, sections.iter());
                sections.push_back(section);
            }
            4 => {
                let section = render_list_of_figures(&mut idg, &mut arena, random_section_id(&sections));
                sections.push_back(section);
            }
            5 => {
                let section = render_index(&mut idg, &mut arena, sections.iter());
                sections.push_back(section);
            }
            6 => {
                let section = render_afterword(&mut idg, &mut arena, || random_section_id(&sections));
                sections.push_back(section);
            }
            _ => unreachable!(),
        }
    }
    sections
}

fn main() {
    let mut result = String::new();
    for (i, section) in generate(50_000).into_iter().enumerate() {
        if i != 0 {
            result.push_str("\n\n");
        }
        result.push_str(&section.content.content);
    }
    println!("{}", result);
}
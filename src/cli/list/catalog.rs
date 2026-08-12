//! Typed inventory behind human and machine-readable command discovery.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub(super) struct EntityCapability {
    pub name: &'static str,
    pub searchable: bool,
    pub gettable: bool,
    pub sections: &'static [&'static str],
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct Placeholder {
    pub name: String,
    #[serde(rename = "type")]
    pub value_type: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(super) enum CatalogEntry {
    Literal {
        section: &'static str,
        command: String,
    },
    Template {
        section: &'static str,
        template: String,
        placeholders: Vec<Placeholder>,
    },
    Prose {
        section: &'static str,
        text: String,
    },
}

const ENTITY_FLAGS: &[(&str, bool, bool)] = &[
    ("gene", true, true),
    ("variant", true, true),
    ("article", true, true),
    ("author", true, true),
    ("trial", true, true),
    ("diagnostic", true, true),
    ("drug", true, true),
    ("disease", true, true),
    ("phenotype", true, false),
    ("pgx", true, true),
    ("gwas", true, false),
    ("pathway", true, true),
    ("protein", true, true),
    ("adverse-event", true, true),
    ("study", false, false),
];

pub(super) fn entities() -> Vec<EntityCapability> {
    ENTITY_FLAGS
        .iter()
        .map(|&(name, searchable, gettable)| EntityCapability {
            name,
            searchable,
            gettable,
            sections: sections(name),
        })
        .collect()
}

pub(crate) fn sections(name: &str) -> &'static [&'static str] {
    match name {
        "gene" => crate::entities::gene::GENE_SECTION_NAMES,
        "variant" => crate::entities::variant::VARIANT_SECTION_NAMES,
        "article" => crate::entities::article::ARTICLE_SECTION_NAMES,
        "trial" => crate::entities::trial::TRIAL_SECTION_NAMES,
        "diagnostic" => crate::entities::diagnostic::DIAGNOSTIC_SECTION_NAMES,
        "drug" => crate::entities::drug::DRUG_SECTION_NAMES,
        "disease" => crate::entities::disease::DISEASE_SECTION_NAMES,
        "pgx" => crate::entities::pgx::PGX_SECTION_NAMES,
        "pathway" => crate::entities::pathway::PATHWAY_SECTION_NAMES,
        "protein" => crate::entities::protein::PROTEIN_SECTION_NAMES,
        "adverse-event" => crate::entities::adverse_event::ADVERSE_EVENT_SECTION_NAMES,
        _ => &[],
    }
}

pub(super) fn entries(page: Option<&str>) -> Vec<CatalogEntry> {
    let mut output = vec![prose(
        "routing",
        "Use literal commands directly. Replace every typed placeholder in a template before execution.",
    )];
    match page {
        None => output.extend(root_entries()),
        Some("skill") => output.extend(skill_entries()),
        Some("study") => output.extend(study_entries()),
        Some("discover") => output.extend(one_surface("discover BRCA1", "discover <query>")),
        Some("search-all") => output.extend(one_surface(
            "search all --gene BRAF",
            "search all --keyword <query>",
        )),
        Some("batch") => output.extend(one_surface("batch gene BRAF,TP53", "batch <entity> <ids>")),
        Some("enrich") => output.extend(one_surface("enrich BRAF,KRAS,NRAS", "enrich <genes>")),
        Some(entity) => output.extend(entity_entries(entity)),
    }
    output
}

fn entity_entries(name: &str) -> Vec<CatalogEntry> {
    let Some(capability) = entities().into_iter().find(|entity| entity.name == name) else {
        return Vec::new();
    };
    let mut output = Vec::new();
    if capability.searchable {
        output.push(template("commands", &search_template(name)));
    }
    if capability.gettable {
        let identifier = if name == "gene" { "symbol" } else { "id" };
        output.push(template("commands", &format!("get {name} <{identifier}>")));
    }
    if name == "variant" {
        output.extend([
            template("helpers", "variant articles <id> --strategy <strategy>"),
            template("helpers", "variant structure <variant>"),
            template("helpers", "variant normalize <service> <transcript_hgvs>"),
        ]);
    }
    output
}

fn search_template(name: &str) -> String {
    match name {
        "article" => "search article --keyword <query>".into(),
        "author" => "search author --query <name> --source semanticscholar".into(),
        "trial" => "search trial --condition <query>".into(),
        "diagnostic" => "search diagnostic --gene <symbol>".into(),
        "pgx" => "search pgx --gene <symbol>".into(),
        "gwas" => "search gwas --gene <symbol>".into(),
        "adverse-event" => "search adverse-event --drug <name>".into(),
        _ => format!("search {name} <query>"),
    }
}

fn root_entries() -> Vec<CatalogEntry> {
    vec![
        literal("quickstart", "get gene BRAF"),
        literal("quickstart", "search trial --condition melanoma"),
        literal("quickstart", "search all --gene BRAF --disease melanoma"),
        literal(
            "quickstart",
            "study top-mutated --study msk_impact_2017 --limit 10",
        ),
        template("patterns", "search <entity> <query>"),
        template("patterns", "get <entity> <id>"),
        template("patterns", "discover <query>"),
    ]
}

fn study_entries() -> Vec<CatalogEntry> {
    vec![
        literal("commands", "study list"),
        literal(
            "commands",
            "study top-mutated --study msk_impact_2017 --limit 10",
        ),
        template("commands", "study download <study_id>"),
        template("commands", "study top-mutated --study <study_id>"),
    ]
}

fn skill_entries() -> Vec<CatalogEntry> {
    vec![
        literal("commands", "skill list"),
        literal("commands", "skill render"),
        literal("commands", "skill 01"),
        template("commands", "skill <number_or_slug>"),
    ]
}

fn one_surface(example: &str, pattern: &str) -> Vec<CatalogEntry> {
    vec![literal("commands", example), template("commands", pattern)]
}

fn literal(section: &'static str, command: &str) -> CatalogEntry {
    CatalogEntry::Literal {
        section,
        command: command.into(),
    }
}

fn prose(section: &'static str, text: &str) -> CatalogEntry {
    CatalogEntry::Prose {
        section,
        text: text.into(),
    }
}

fn template(section: &'static str, value: &str) -> CatalogEntry {
    let placeholders = value
        .split('<')
        .skip(1)
        .filter_map(|tail| tail.split_once('>').map(|(name, _)| name))
        .map(|name| Placeholder {
            name: name.into(),
            value_type: placeholder_type(name),
        })
        .collect();
    CatalogEntry::Template {
        section,
        template: value.into(),
        placeholders,
    }
}

fn placeholder_type(name: &str) -> &'static str {
    match name {
        "entity" => "entity",
        "id" => "identifier",
        "ids" => "comma_separated_identifiers",
        "symbol" => "gene_symbol",
        "study_id" => "study_identifier",
        "genes" => "comma_separated_gene_symbols",
        "strategy" => "enum:union|annotation|lexical",
        "number_or_slug" => "skill_identifier",
        "service" => "enum:all|mutalyzer|variantvalidator",
        "transcript_hgvs" => "transcript_hgvs",
        "variant" => "variant",
        "name" | "query" => "text",
        _ => "text",
    }
}

pub(super) fn render_entity_inventory() -> String {
    let entities = entities();
    let mut output = String::from("## Gettable Entities\n\n");
    for entity in entities.iter().filter(|entity| entity.gettable) {
        output.push_str(&format!("- {}\n", entity.name));
    }
    output.push_str("\n## Search-Only Entities\n\n");
    for entity in entities
        .iter()
        .filter(|entity| entity.searchable && !entity.gettable)
    {
        let description = match entity.name {
            "gwas" => "GWAS Catalog",
            "phenotype" => "Monarch/HPO disease similarity",
            _ => "search-only entity",
        };
        output.push_str(&format!(
            "- `{}` - {}; use `search {}`\n",
            entity.name, description, entity.name
        ));
    }
    output.push_str("\n## Other Catalog Pages\n\n");
    output.push_str("- `study` - local cBioPortal analytics; use `study list`\n");
    output
}

pub(super) fn validate(entries: &[CatalogEntry]) -> Result<(), crate::error::BioMcpError> {
    for entry in entries {
        let CatalogEntry::Literal { command, .. } = entry else {
            continue;
        };
        let split = shlex::split(command).ok_or_else(|| {
            crate::error::BioMcpError::InvalidArgument(format!(
                "Invalid catalog command syntax: {command}"
            ))
        })?;
        let args: Vec<_> = std::iter::once("biomcp".to_string()).chain(split).collect();
        crate::cli::try_parse_cli(args).map_err(|error| {
            crate::error::BioMcpError::InvalidArgument(format!(
                "Invalid catalog command `{command}`: {error}"
            ))
        })?;
    }
    Ok(())
}

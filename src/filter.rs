use crate::parser::Entry;
use regex::Regex;
use std::sync::LazyLock;

fn re(pattern: &str) -> Regex {
    match Regex::new(pattern) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[FATAL] regex compile failed: {e}");
            std::process::exit(1);
        }
    }
}

static ARXIV_DOMAIN_ANCHORS: LazyLock<Regex> = LazyLock::new(|| {
    re(concat!(
        r"(?i)(?:^|\W)(?:",
        r"large[\s\-]?language[\s\-]?model\w*|\bllm\w*\b|transformer\w*|\bgpt\w*\b|claude",
        r"|language[\s\-]?model\w*|neural\w*|deep[\s\-]?learn\w*|machine[\s\-]?learn\w*",
        r"|software[\s\-]?engineer\w*|code[\s\-]?generat\w*|program[\s\-]?synth\w*",
        r"|natural[\s\-]?language|\bnlp\b|chatbot\w*|conversational",
        r"|autonomous[\s\-]?agent|multi[\s\-]?agent|ai[\s\-]?agent",
        r"|reinforcement[\s\-]?learn\w*|\brlhf\b|reward[\s\-]?model",
        r"|diffusion[\s\-]?model|generative[\s\-]?ai|foundation[\s\-]?model",
        r"|benchmark\w*|evaluat\w+[\s\-]?(?:model|llm|agent)",
        r"|instruction[\s\-]?(?:tun|follow)\w*|in[\s\-]?context[\s\-]?learn\w*",
        r")",
    ))
});

static ARXIV_TECH_SIGNALS: LazyLock<Regex> = LazyLock::new(|| {
    re(concat!(
        r"(?i)(?:^|\W)(?:",
        r"retriev\w*[\s\-]+augment\w*[\s\-]+generat\w*",
        r"|rag[\s\-]?(?:system|pipeline|framework|retriev|chunk|vector|survey|code|approach)\w*",
        r"|ai[\s\-]?(?:align|safe)\w*",
        r"|align\w*[\s\-]+(?:llm|model|language|rlhf|preference|human)\w*",
        r"|red[\s\-]?team\w*|jailbreak\w*|guardrail\w*|safety[\s\-]?filter\w*",
        r"|agentic\w*|vibe[\s\-]?cod\w*|ai[\s\-]?cod\w*|coding[\s\-]?agent\w*|code[\s\-]?agent\w*",
        r"|claude[\s\-]?code|cursor[\s\-]?ai|copilot[\s\-]?(?:agent|chat|x)\w*",
        r"|context[\s\-]?engineer\w*|prompt[\s\-]?engineer\w*",
        r"|\bmcp\b|model[\s\-]?context[\s\-]?protocol",
        r"|tool[\s\-]?use|function[\s\-]?call\w*|tool[\s\-]?augment\w*",
        r"|fine[\s\-]?tun\w*|\blora\b|\bqlora\b|adapter\w*",
        r"|swe[\s\-]?bench|swe[\s\-]?agent|code[\s\-]?review",
        r"|chain[\s\-]?of[\s\-]?thought|\bcot\b|reason\w+[\s\-]?model",
        r"|tokeniz\w*|embed\w+[\s\-]?model|vector[\s\-]?(?:db|store|search)",
        r"|ai[\s\-]?pair|code[\s\-]?complet\w*|code[\s\-]?assist\w*",
        r"|\brust\b|\bcargo\b|\btokio\b|\bwasm\b",
        r")",
    ))
});

static ARXIV_NEGATIVE: LazyLock<Regex> = LazyLock::new(|| {
    re(concat!(
        r"(?i)(?:^|\W)(?:",
        r"medical|clinical|biomedic\w*|patholog\w*|radiology|diagnosis",
        r"|patient\w*|disease\w*|drug\w*|pharma\w*|health[\s\-]?care",
        r"|legal|jurisprud\w*|courtroom|litigation",
        r"|biolog\w*|genomic\w*|protein\w*|molecul\w*|\bdna\b|\brna\b|gene[\s\-]?express\w*",
        r"|astrono\w*|cosmolog\w*|astrophys\w*|galaxy|stellar",
        r"|quantum[\s\-]?(?:comput|mechan|field)\w*",
        r"|fluid[\s\-]?dynam\w*|thermodynam\w*",
        r"|seismic\w*|geolog\w*|climate[\s\-]?model",
        r"|optical[\s\-]?align|beam[\s\-]?align|laser[\s\-]?align",
        r"|sequence[\s\-]?align|structural[\s\-]?align",
        r"|crystal\w*|lattice\w*|phonon\w*",
        r")",
    ))
});

static ARXIV_CATEGORY_RE: LazyLock<Regex> =
    LazyLock::new(|| re(r"\b((?:cs|stat)\.\w{2})\b"));

const ARXIV_ALLOWED_CATEGORIES: &[&str] = &[
    "cs.AI", "cs.CL", "cs.SE", "cs.LG", "cs.IR", "cs.MA", "cs.HC", "cs.CR", "cs.PL", "cs.FL",
    "stat.ML",
];

pub fn is_arxiv_source(name: &str) -> bool {
    name.to_lowercase().starts_with("arxiv")
}

pub fn passes_keyword_filter(entry: &Entry, keywords_re: &Regex) -> bool {
    let text = format!("{} {}", entry.title, entry.desc);
    keywords_re.is_match(&text)
}

pub fn passes_arxiv_filter(entry: &Entry) -> bool {
    let text = format!("{} {}", entry.title, entry.desc);

    if ARXIV_NEGATIVE.is_match(&text) {
        return false;
    }
    if !(ARXIV_DOMAIN_ANCHORS.is_match(&text) && ARXIV_TECH_SIGNALS.is_match(&text)) {
        return false;
    }
    arxiv_category_ok(entry)
}

fn arxiv_category_ok(entry: &Entry) -> bool {
    let text = format!("{} {} {}", entry.link, entry.desc, entry.title);
    let categories: Vec<&str> = ARXIV_CATEGORY_RE
        .find_iter(&text)
        .map(|m| m.as_str())
        .collect();
    if categories.is_empty() {
        return true;
    }
    categories
        .iter()
        .any(|cat| ARXIV_ALLOWED_CATEGORIES.contains(cat))
}

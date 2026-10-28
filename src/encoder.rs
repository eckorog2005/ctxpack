use std::sync::OnceLock;

use tiktoken_rs::CoreBPE;

#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub enum TokenizerKind {
    O200k,
    Cl100k,
}

pub struct Encoder {
    bpe: CoreBPE,
    name: &'static str,
}

static O200K: OnceLock<Encoder> = OnceLock::new();
static CL100K: OnceLock<Encoder> = OnceLock::new();

impl Encoder {
    /// The first call per kind builds the BPE, the largest fixed startup cost.
    pub fn get(kind: TokenizerKind) -> &'static Encoder {
        match kind {
            TokenizerKind::O200k => O200K.get_or_init(|| Encoder {
                bpe: tiktoken_rs::o200k_base().expect("o200k_base ranks are embedded"),
                name: "o200k_base",
            }),
            TokenizerKind::Cl100k => CL100K.get_or_init(|| Encoder {
                bpe: tiktoken_rs::cl100k_base().expect("cl100k_base ranks are embedded"),
                name: "cl100k_base",
            }),
        }
    }

    /// Special tokens such as `<|endoftext|>` are counted as plain text.
    pub fn count(&self, text: &str) -> usize {
        self.bpe.encode_ordinary(text).len()
    }

    pub fn name(&self) -> &'static str {
        self.name
    }
}

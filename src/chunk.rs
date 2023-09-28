use std::collections::VecDeque;

use index_vec::IndexVec;

use crate::{span::Span, CowStr, TextSize};

index_vec::define_index_type! {
    pub struct ChunkIdx = u32;
}

pub type ChunkVec<'s> = IndexVec<ChunkIdx, Chunk<'s>>;

#[derive(Debug)]
pub struct EditOptions {
    /// `true` will clear the `intro` and `outro` of the [Chunk]
    pub overwrite: bool,
    pub store_name: bool,
}

impl Default for EditOptions {
    fn default() -> Self {
        Self {
            overwrite: true,
            store_name: false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Chunk<'str> {
    pub intro: VecDeque<CowStr<'str>>,
    pub outro: VecDeque<CowStr<'str>>,
    pub span: Span,
    pub edited_content: Option<CowStr<'str>>,
    pub(crate) next: Option<ChunkIdx>,
    pub store_name: bool,
}

impl<'s> Chunk<'s> {
    pub fn new(span: Span) -> Self {
        debug_assert!(span.0 < span.1);
        Self {
            span,
            ..Default::default()
        }
    }
}

impl<'str> Chunk<'str> {
    pub fn start(&self) -> TextSize {
        self.span.start()
    }

    pub fn end(&self) -> TextSize {
        self.span.end()
    }

    pub fn contains(&self, text_index: TextSize) -> bool {
        self.start() < text_index && text_index < self.end()
    }

    pub fn append_outro(&mut self, content: CowStr<'str>) {
        self.outro.push_back(content)
    }

    pub fn append_intro(&mut self, content: CowStr<'str>) {
        self.intro.push_back(content)
    }

    pub fn prepend_outro(&mut self, content: CowStr<'str>) {
        self.outro.push_front(content)
    }

    pub fn prepend_intro(&mut self, content: CowStr<'str>) {
        self.intro.push_front(content)
    }

    pub fn split<'a>(&'a mut self, text_index: TextSize) -> Chunk<'str> {
        debug_assert!(text_index > self.start());
        debug_assert!(text_index < self.end());
        if self.edited_content.is_some() {
            panic!("Cannot split a chunk that has already been edited")
        }
        let first_slice_span = Span(self.start(), text_index);
        let last_slice_span = Span(text_index, self.end());
        let mut new_chunk = Chunk::new(last_slice_span);
        if self.is_edited() {
            new_chunk.edit("".into(), Default::default());
        }
        std::mem::swap(&mut new_chunk.outro, &mut self.outro);
        self.span = first_slice_span;
        new_chunk.next = self.next;
        new_chunk
    }

    pub fn fragments(
        &'str self,
        original_source: &'str CowStr<'str>,
    ) -> impl Iterator<Item = &'str str> {
        let intro_iter = self.intro.iter().map(|frag| frag.as_ref());
        let source_frag = self
            .edited_content
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or_else(|| self.span.text(original_source.as_str()));
        let outro_iter = self.outro.iter().map(|frag| frag.as_ref());
        intro_iter.chain(Some(source_frag)).chain(outro_iter)
    }

    pub fn edit(&mut self, content: CowStr<'str>, opts: EditOptions) {
        if opts.overwrite {
            self.intro.clear();
            self.outro.clear();
        }
        self.store_name = opts.store_name;
        self.edited_content = Some(content);
    }

    pub fn is_edited(&self) -> bool {
        self.edited_content.is_some()
    }
}

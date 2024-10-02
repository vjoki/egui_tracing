use std::borrow::Cow;

use unicode_segmentation::UnicodeSegmentation;

pub trait Ellipse {
    fn truncate_graphemes(&self, len: usize) -> Cow<'_, str>;
}

macro_rules! impl_truncate_graphemes {
    ($ty:ty, $self:ident $cowself:block) => {
        impl Ellipse for $ty {
            fn truncate_graphemes(&$self, len: usize) -> Cow<'_, str> {
                if $self.len() <= len {
                    return Cow::from($cowself);
                }

                let mut truncated = String::with_capacity(len + 3);
                truncated.extend($self.graphemes(true).take(len));
                truncated.push_str("...");

                Cow::from(truncated)
            }
        }
    };
}

impl_truncate_graphemes!(String, self { self });
impl_truncate_graphemes!(&str, self { *self });

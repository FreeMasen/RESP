use ress::{Item, RefToken};

/// A comment handler will allow you to specify
/// behavior about what to do with comments
/// officially comments are supposed to operate
/// the same as whitespace so the default behavior
/// would be to throw away any comments found
pub trait CommentHandler<'a> {
    fn handle_comment(&mut self, comment: Item<RefToken<'a>>);
}
/// The default comment handler,
/// this will discard comments
/// provided to it
pub struct DefaultCommentHandler;

impl<'a> CommentHandler<'a> for DefaultCommentHandler {
    fn handle_comment(&mut self, _: Item<RefToken<'a>>) {}
}

impl<'a, F> CommentHandler<'a> for F
where
    F: FnMut(Item<RefToken<'a>>),
{
    fn handle_comment(&mut self, item: Item<RefToken<'a>>) {
        self(item)
    }
}

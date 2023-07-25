use lofty::{Accessor, TaggedFileExt};

pub fn list(song: &lofty::TaggedFile) -> Option<String> {
    let Some(tag) = song.tag(song.primary_tag_type()) else {
            return None
                        };
    let title = if let Some(title) = tag.title() {
        format!("'{title}'")
    } else {
        String::from("Untitled Song")
    };
    let artist = if let Some(artist) = tag.artist() {
        format!(" by {artist}")
    } else {
        Default::default()
    };
    let album = if let Some(album) = tag.album() {
        format!(" in album {album}")
    } else {
        Default::default()
    };
    Some(format!("{title}{artist}{album}"))
}

use anyhow::Result;
use lofty::TaggedFileExt;
use lofty::{Accessor, TaggedFile};

pub fn update(songs: Vec<TaggedFile>) -> Result<()> {
    for song in songs {
        let tag = song.tag(song.primary_tag_type()).expect("song lacks a tag");
        let album = if let Some(album) = tag.album() {
            format!(" in album '{album}'")
        } else {
            Default::default()
        };
        let artist = tag.artist().unwrap_or_default();
        let title = tag.title().unwrap_or_default();
        println!("'{title}' by {artist}{album}");
    }
    Ok(())
}
